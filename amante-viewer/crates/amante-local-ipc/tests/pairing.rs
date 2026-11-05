use amante_local_ipc::pairing::{self, Listener};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

fn fixture() -> (std::net::SocketAddr, Arc<AtomicBool>, thread::JoinHandle<usize>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let worker = stop.clone();
    let handle = thread::spawn(move || {
        let mut listener = Listener::new(listener, "ab".repeat(32)).unwrap();
        let mut accepted = 0;
        while !worker.load(Ordering::Acquire) {
            if let Some(mut stream) = listener.poll().unwrap() {
                accepted += 1;
                stream.write_all(b"authorized\n").unwrap();
            }
            thread::sleep(Duration::from_millis(2));
        }
        accepted
    });
    (addr, stop, handle)
}

fn authorized(addr: std::net::SocketAddr) {
    let mut stream = TcpStream::connect(addr).unwrap();
    pairing::connect(&mut stream, &"ab".repeat(32)).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut reply = [0; 11];
    stream.read_exact(&mut reply).unwrap();
    assert_eq!(&reply, b"authorized\n");
}

#[test]
fn amante_security_competing_silent_wrong_legacy_and_reconnecting_clients() {
    let (addr, stop, worker) = fixture();
    let _silent: Vec<_> = (0..8).map(|_| TcpStream::connect(addr).unwrap()).collect();
    let mut legacy = TcpStream::connect(addr).unwrap();
    legacy.write_all(pairing::MAGIC).unwrap();
    let mut wrong = TcpStream::connect(addr).unwrap();
    assert!(pairing::connect(&mut wrong, &"cd".repeat(32)).is_err());
    authorized(addr);
    authorized(addr);
    stop.store(true, Ordering::Release);
    assert_eq!(worker.join().unwrap(), 2);
}

#[test]
fn amante_security_candidate_capacity_expiry_and_pairing_rotation() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    let mut gate = Listener::new(listener, "ab".repeat(32)).unwrap();
    let _clients: Vec<_> = (0..16).map(|_| TcpStream::connect(addr).unwrap()).collect();
    assert!(gate.poll().unwrap().is_none());
    let mut excess = TcpStream::connect(addr).unwrap();
    excess.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    assert!(gate.poll().unwrap().is_none());
    assert!(matches!(excess.read(&mut [0]), Ok(0) | Err(_)));
    thread::sleep(pairing::TIMEOUT + Duration::from_millis(20));
    assert!(gate.poll().unwrap().is_none());
    // A new session rejects proof using the previous session's code.
    drop(gate);
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        pairing::accept(&mut stream, &pairing::new_code(), &AtomicBool::new(false)).is_err()
    });
    let mut client = TcpStream::connect(addr).unwrap();
    assert!(pairing::connect(&mut client, &"ab".repeat(32)).is_err());
    drop(client);
    assert!(worker.join().unwrap());
}

#[test]
fn amante_security_absolute_deadline_and_stop_interrupt_slow_bytes() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let _client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (mut stream, _) = listener.accept().unwrap();
    stream.set_read_timeout(Some(Duration::from_millis(20))).unwrap();
    let start = Instant::now();
    let error = pairing::read_deadline(
        &mut stream,
        &mut [0; 10],
        &AtomicBool::new(false),
        start + Duration::from_millis(60),
    )
    .unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(
        pairing::read_deadline(&mut stream, &mut [0], &AtomicBool::new(true), Instant::now() + pairing::TIMEOUT)
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::Interrupted
    );
}

#[test]
fn amante_security_replay_and_reflected_server_proof_are_rejected() {
    let (addr, stop, worker) = fixture();
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let hello = [pairing::MAGIC.as_slice(), &[0; 32]].concat();
    stream.write_all(&hello).unwrap();
    let mut first = [0; 64];
    stream.read_exact(&mut first).unwrap();
    // A server's proof must not double as client authorization.
    stream.write_all(&first[32..]).unwrap();
    assert!(matches!(stream.read(&mut [0]), Ok(0) | Err(_)));
    let mut replay = TcpStream::connect(addr).unwrap();
    replay.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    replay.write_all(&hello).unwrap();
    let mut next = [0; 64];
    replay.read_exact(&mut next).unwrap();
    assert_ne!(first, next);
    replay.write_all(&first[32..]).unwrap();
    assert!(matches!(replay.read(&mut [0]), Ok(0) | Err(_)));
    stop.store(true, Ordering::Release);
    assert_eq!(worker.join().unwrap(), 0);
}

#[test]
fn amante_security_python_addon_pairs_with_real_rust_gate() {
    let (addr, stop, worker) = fixture();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../blender/aurora_live/client.py");
    // Python is also used by the Blender addon, without importing bpy here.
    let output = std::process::Command::new(if cfg!(windows) { "python" } else { "python3" })
        .arg("-c")
        .arg("import importlib.util,socket,sys\ns=importlib.util.spec_from_file_location('client',sys.argv[1]); m=importlib.util.module_from_spec(s); s.loader.exec_module(m)\nwith socket.create_connection(('127.0.0.1',int(sys.argv[2]))) as c:\n m.pair(c,'ab'*32)\n assert c.recv(11)==b'authorized\\n'\n")
        .arg(path).arg(addr.port().to_string()).output().unwrap();
    stop.store(true, Ordering::Release);
    let count = worker.join().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(count, 1);
}
