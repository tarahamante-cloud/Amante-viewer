//! Ephemeral TLS, synthetic credentials, loopback only; never a live grid.

use super::*;
use crate::login::{LastExec, LoginRequest, StartLocation};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};

struct Server {
    address: SocketAddr,
    certificate: reqwest::Certificate,
    requests: Arc<Mutex<Vec<String>>>,
    task: JoinHandle<()>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Server {
    async fn start(replies: impl FnOnce(SocketAddr) -> Vec<String>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let rcgen::CertifiedKey { cert, signing_key } = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let certificate = reqwest::Certificate::from_der(cert.der()).unwrap();
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::aws_lc_rs::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(
                vec![cert.der().clone()],
                rustls::pki_types::PrivatePkcs8KeyDer::from(signing_key.serialize_der()).into(),
            )
            .unwrap();
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        let replies = replies(address);
        let task = tokio::spawn(async move {
            for response in replies {
                let (stream, _) = listener.accept().await.unwrap();
                let Ok(mut stream) = acceptor.accept(stream).await else {
                    continue;
                };
                let mut data = Vec::new();
                loop {
                    let mut buf = [0; 4096];
                    let n = stream.read(&mut buf).await.unwrap();
                    if n == 0 {
                        break;
                    }
                    data.extend_from_slice(&buf[..n]);
                    assert!(data.len() < 64 * 1024);
                    let request = String::from_utf8_lossy(&data);
                    if let Some(end) = request.find("\r\n\r\n") {
                        let length = request[..end]
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|n| n.trim().parse::<usize>().unwrap())
                            })
                            .unwrap_or(0);
                        if data.len() >= end + 4 + length {
                            break;
                        }
                    }
                }
                recorded.lock().unwrap().push(String::from_utf8(data).unwrap());
                stream.write_all(response.as_bytes()).await.unwrap();
                let _ = stream.shutdown().await;
            }
        });
        Self {
            address,
            certificate,
            requests,
            task,
        }
    }

    fn url(&self) -> String {
        format!("https://localhost:{}/login", self.address.port())
    }

    fn client(&self, builder: ClientBuilder) -> Client {
        builder
            .tls_certs_only([self.certificate.clone()])
            .resolve("localhost", self.address)
            .resolve("wrong.invalid", self.address)
            .no_proxy()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap()
    }
}

fn response(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn redirect_xml(next: &str) -> String {
    response(&format!(
        "<methodResponse><params><param><value><struct>\
         <member><name>login</name><value><string>indeterminate</string></value></member>\
         <member><name>next_url</name><value><string>{next}</string></value></member>\
         </struct></value></param></params></methodResponse>"
    ))
}

fn request(uri: String) -> LoginRequest {
    LoginRequest {
        login_uri: uri,
        username: "synthetic-account".into(),
        password: "SYNTHETIC_PASSWORD_NOT_A_CREDENTIAL".into(),
        start: StartLocation::Last,
        agree_to_tos: false,
        read_critical: false,
        mfa_token: "SYNTHETIC_OTP".into(),
        mfa_hash: "SYNTHETIC_MFA_AUTHORIZATION".into(),
        mac: "synthetic-device".into(),
        id0: "synthetic-device".into(),
        last_exec: LastExec::default(),
    }
}

#[tokio::test]
async fn amante_security_trusted_certificate_still_requires_matching_hostname() {
    for (host, accepted) in [("localhost", true), ("wrong.invalid", false)] {
        let server = Server::start(|_| vec![response("ok")]).await;
        let client = server.client(simulator_client());
        let result = client
            .post(format!("https://{host}:{}/", server.address.port()))
            .body("synthetic secret")
            .send()
            .await;
        assert_eq!(result.is_ok(), accepted);
        assert_eq!(server.requests.lock().unwrap().len(), usize::from(accepted));
    }
}

#[tokio::test]
async fn amante_security_xmlrpc_and_http_redirects_send_nothing_to_rejected_origin() {
    for xmlrpc in [true, false] {
        let rejected = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let target = format!("https://localhost:{}/rejected", rejected.local_addr().unwrap().port());
        let server = Server::start(|_| {
            vec![if xmlrpc {
                redirect_xml(&target)
            } else {
                format!("HTTP/1.1 307 Temporary Redirect\r\nLocation: {target}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            }]
        })
        .await;
        let client = server.client(login_client());
        assert!(crate::login::login(&client, &request(server.url())).await.is_err());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
        assert!(tokio::time::timeout(Duration::from_millis(100), rejected.accept()).await.is_err());
    }
}

#[tokio::test]
async fn amante_security_same_origin_xmlrpc_redirect_remains_supported() {
    let refusal = response(
        "<methodResponse><params><param><value><struct>\
         <member><name>login</name><value><string>false</string></value></member>\
         <member><name>reason</name><value><string>synthetic-end</string></value></member>\
         </struct></value></param></params></methodResponse>",
    );
    let server = Server::start(|addr| vec![redirect_xml(&format!("https://localhost:{}/next", addr.port())), refusal]).await;
    let result = crate::login::login(&server.client(login_client()), &request(server.url())).await;
    assert!(matches!(result, Err(crate::login::LoginError::Refused { reason, .. }) if reason == "synthetic-end"));
    assert_eq!(server.requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn amante_security_plaintext_login_is_rejected_before_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let req = request(format!("http://{}/login", listener.local_addr().unwrap()));
    assert!(crate::login::login(&Client::new(), &req).await.is_err());
    assert!(tokio::time::timeout(Duration::from_millis(100), listener.accept()).await.is_err());
}

#[test]
fn amante_security_login_debug_does_not_disclose_secrets() {
    let req = request("https://localhost/login?synthetic-session".into());
    let debug = format!("{req:?}");
    for secret in [&req.username, &req.password, &req.mfa_token, &req.mfa_hash, &req.login_uri] {
        assert!(!debug.contains(secret));
    }
}

#[tokio::test]
async fn amante_security_capability_posts_never_replay_to_another_origin_or_plaintext() {
    let rejected = TcpListener::bind("127.0.0.1:0").await.unwrap();
    for scheme in ["https", "http"] {
        let target = format!("{scheme}://localhost:{}/rejected", rejected.local_addr().unwrap().port());
        let server = Server::start(|_| {
            vec![format!(
                "HTTP/1.1 307 Temporary Redirect\r\nLocation: {target}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            )]
        })
        .await;
        let result = server
            .client(simulator_client())
            .post(server.url())
            .body("SYNTHETIC_SESSION")
            .send()
            .await;
        assert!(result.is_err());
        assert_eq!(server.requests.lock().unwrap().len(), 1);
        assert!(tokio::time::timeout(Duration::from_millis(100), rejected.accept()).await.is_err());
    }
    let url = format!("http://{}/cap/synthetic", rejected.local_addr().unwrap());
    assert!(
        simulator_client()
            .no_proxy()
            .build()
            .unwrap()
            .post(url)
            .body("SYNTHETIC_SESSION")
            .send()
            .await
            .is_err()
    );
    assert!(tokio::time::timeout(Duration::from_millis(100), rejected.accept()).await.is_err());
}

#[test]
fn amante_security_login_origin_policy_rejects_lookalikes_and_downgrades() {
    let initial = Url::parse("https://login.example/login").unwrap();
    assert!(credential_destination(
        &initial,
        &Url::parse("https://LOGIN.example:443/next").unwrap()
    ));
    for target in [
        "http://login.example/next",
        "https://login.example:444/next",
        "https://login.example.evil.invalid/next",
        "https://login.example@evil.invalid/next",
        "https://user:password@login.example/next",
        "https://login.example/next#fragment",
    ] {
        assert!(!credential_destination(&initial, &Url::parse(target).unwrap()), "{target}");
    }
}
