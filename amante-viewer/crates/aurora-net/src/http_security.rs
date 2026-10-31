//! Shared TLS and credential redirect rules. Keeping Linden's CA does not
//! authorize a certificate for a different hostname.

use reqwest::{Client, ClientBuilder, Url, redirect::Policy};

pub(crate) fn login_client() -> ClientBuilder {
    let ll_ca = reqwest::Certificate::from_pem_bundle(include_bytes!("../lindenlab.pem")).unwrap_or_default();
    Client::builder()
        .https_only(true)
        .user_agent(aurora_version::user_agent())
        .connect_timeout(std::time::Duration::from_secs(15))
        .tls_certs_merge(ll_ca)
        .redirect(login_redirects())
}

pub(crate) fn simulator_client() -> ClientBuilder {
    Client::builder()
        .https_only(true)
        .user_agent(aurora_version::user_agent())
        .tls_certs_only(crate::simulator_roots())
        .redirect(login_redirects())
}

/// Only the explicitly selected HTTPS origin is trusted with login secrets.
/// A server response cannot authorize a new host, port, or protocol.
pub fn credential_destination(initial: &Url, target: &Url) -> bool {
    initial.scheme() == "https"
        && target.scheme() == "https"
        && initial.host_str().is_some()
        && target.host_str().is_some()
        && initial.username().is_empty()
        && initial.password().is_none()
        && target.username().is_empty()
        && target.password().is_none()
        && initial.fragment().is_none()
        && target.fragment().is_none()
        && initial.origin() == target.origin()
}

pub fn login_redirects() -> Policy {
    Policy::custom(|attempt| {
        if attempt.previous().len() >= 5 {
            attempt.error("too many login redirects")
        } else if attempt
            .previous()
            .first()
            .is_some_and(|initial| credential_destination(initial, attempt.url()))
        {
            attempt.follow()
        } else {
            // Never include the URL: it may contain a reusable session token.
            attempt.error("untrusted login redirect")
        }
    })
}

#[cfg(test)]
mod tests;
