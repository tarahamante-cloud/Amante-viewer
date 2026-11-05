//! Fixed-size, mutually authenticated prelude to AURBLD1. Session keys never
//! travel on the wire. Role-separated HMACs bind fresh nonces against replay.
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

pub const MAGIC: &[u8; 8] = b"AURBLD1\n";
pub const TIMEOUT: Duration = Duration::from_secs(2);

pub fn new_code() -> String {
    format!("{}{}", uuid::Uuid::new_v4().simple(), uuid::Uuid::new_v4().simple())
}

fn nonce() -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    bytes[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    bytes
}

fn mac(code: &str, role: &[u8], client: &[u8], server: &[u8]) -> Hmac<Sha256> {
    let mut mac = Hmac::<Sha256>::new_from_slice(code.as_bytes()).expect("HMAC accepts any key length");
    mac.update(b"Amante Blender pairing v1\0");
    mac.update(role);
    mac.update(client);
    mac.update(server);
    mac
}

/// Absolute deadline, not an idle timeout: byte-at-a-time senders cannot
/// indefinitely retain either authentication or a large frame allocation.
pub fn read_deadline(stream: &mut TcpStream, mut bytes: &mut [u8], stop: &AtomicBool, deadline: Instant) -> io::Result<()> {
    while !bytes.is_empty() {
        if stop.load(Ordering::Acquire) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if Instant::now() >= deadline {
            return Err(io::ErrorKind::TimedOut.into());
        }
        match stream.read(bytes) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(n) => bytes = &mut bytes[n..],
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
                ) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

pub fn accept(stream: &mut TcpStream, code: &str, stop: &AtomicBool) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_millis(50)))?;
    stream.set_write_timeout(Some(Duration::from_millis(250)))?;
    let deadline = Instant::now() + TIMEOUT;
    let mut hello = [0; 40];
    read_deadline(stream, &mut hello, stop, deadline)?;
    if &hello[..8] != MAGIC {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Unpaired Blender client"));
    }
    let client = &hello[8..];
    let server = nonce();
    stream.write_all(&server)?;
    stream.write_all(&mac(code, b"server", client, &server).finalize().into_bytes())?;
    let mut proof = [0; 32];
    read_deadline(stream, &mut proof, stop, deadline)?;
    mac(code, b"client", client, &server)
        .verify_slice(&proof)
        .map_err(|_| io::Error::new(io::ErrorKind::PermissionDenied, "Unpaired Blender client"))
}

/// Also used by synthetic peers to exercise the real wire protocol.
pub fn connect(stream: &mut TcpStream, code: &str) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_millis(50)))?;
    stream.set_write_timeout(Some(Duration::from_millis(250)))?;
    let stop = AtomicBool::new(false);
    let deadline = Instant::now() + TIMEOUT;
    let client = nonce();
    stream.write_all(MAGIC)?;
    stream.write_all(&client)?;
    let mut reply = [0; 64];
    read_deadline(stream, &mut reply, &stop, deadline)?;
    mac(code, b"server", &client, &reply[..32])
        .verify_slice(&reply[32..])
        .map_err(|_| io::Error::new(io::ErrorKind::PermissionDenied, "Unpaired Blender viewer"))?;
    stream.write_all(&mac(code, b"client", &client, &reply[..32]).finalize().into_bytes())
}

/// Poll a bounded set of candidates concurrently. A silent or slow unpaired
/// connection cannot monopolize the worker ahead of a legitimate addon.
pub struct Listener {
    listener: TcpListener,
    code: String,
    candidates: Vec<Candidate>,
}

struct Candidate {
    stream: TcpStream,
    deadline: Instant,
    hello: [u8; 40],
    reply: [u8; 64],
    proof: [u8; 32],
    stage: u8,
    offset: usize,
}

impl Listener {
    pub fn new(listener: TcpListener, code: String) -> io::Result<Self> {
        listener.set_nonblocking(true)?;
        Ok(Self {
            listener,
            code,
            candidates: Vec::new(),
        })
    }

    pub fn poll(&mut self) -> io::Result<Option<TcpStream>> {
        // Bound accepts as well as candidates/work per tick under a flood.
        for _ in 0..16 {
            match self.listener.accept() {
                Ok((stream, address)) if address.ip().is_loopback() && self.candidates.len() < 16 => {
                    stream.set_nonblocking(true)?;
                    self.candidates.push(Candidate {
                        stream,
                        deadline: Instant::now() + TIMEOUT,
                        hello: [0; 40],
                        reply: [0; 64],
                        proof: [0; 32],
                        stage: 0,
                        offset: 0,
                    });
                }
                Ok(_) => {} // Explicitly close over-capacity/non-loopback peers.
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
        let mut index = 0;
        while index < self.candidates.len() {
            match self.candidates[index].poll(&self.code) {
                Ok(true) => {
                    let candidate = self.candidates.swap_remove(index);
                    self.candidates.clear();
                    candidate.stream.set_nonblocking(false)?;
                    return Ok(Some(candidate.stream));
                }
                Ok(false) => index += 1,
                Err(_) => {
                    // No application state/rig information touched before auth.
                    self.candidates.swap_remove(index);
                }
            }
        }
        Ok(None)
    }
}

impl Candidate {
    fn poll(&mut self, code: &str) -> io::Result<bool> {
        if Instant::now() >= self.deadline {
            return Err(io::ErrorKind::TimedOut.into());
        }
        for _ in 0..3 {
            let result = match self.stage {
                0 => self.stream.read(&mut self.hello[self.offset..]),
                1 => self.stream.write(&self.reply[self.offset..]),
                _ => self.stream.read(&mut self.proof[self.offset..]),
            };
            match result {
                Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
                Ok(n) => self.offset += n,
                Err(e) if matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted) => return Ok(false),
                Err(e) => return Err(e),
            }
            let length = match self.stage {
                0 => 40,
                1 => 64,
                _ => 32,
            };
            if self.offset < length {
                return Ok(false);
            }
            match self.stage {
                0 => {
                    if &self.hello[..8] != MAGIC {
                        return Err(io::ErrorKind::PermissionDenied.into());
                    }
                    self.reply[..32].copy_from_slice(&nonce());
                    let tag = mac(code, b"server", &self.hello[8..], &self.reply[..32]).finalize().into_bytes();
                    self.reply[32..].copy_from_slice(&tag);
                }
                1 => {}
                _ => {
                    mac(code, b"client", &self.hello[8..], &self.reply[..32])
                        .verify_slice(&self.proof)
                        .map_err(|_| io::Error::from(io::ErrorKind::PermissionDenied))?;
                    return Ok(true);
                }
            }
            self.offset = 0;
            self.stage += 1;
        }
        Ok(false)
    }
}
