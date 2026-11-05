use std::{io, net::TcpStream};

/// Authorize only the exact launched process, not other processes of the same
/// user, its children, or an executable with the same filename. Errors deny.
pub fn owns_connection(stream: &TcpStream, pid: u32) -> io::Result<bool> {
    if !stream.peer_addr()?.ip().is_loopback() || !stream.local_addr()?.ip().is_loopback() {
        return Ok(false);
    }
    platform_owns(stream, pid)
}

#[cfg(target_os = "linux")]
fn platform_owns(stream: &TcpStream, pid: u32) -> io::Result<bool> {
    use std::{collections::HashSet, fs, net::SocketAddr};
    let encode = |addr: SocketAddr| -> io::Result<String> {
        let SocketAddr::V4(addr) = addr else {
            return Err(io::ErrorKind::Unsupported.into());
        };
        Ok(format!("{:08X}:{:04X}", u32::from_ne_bytes(addr.ip().octets()), addr.port()))
    };
    // Look up the CLIENT endpoint, then confirm that socket's inode is owned
    // by the still-live Child handle. Reading the PID's own namespace table
    // also works inside network namespaces. No command or shell is invoked.
    let local = encode(stream.peer_addr()?)?;
    let remote = encode(stream.local_addr()?)?;
    let table = fs::read_to_string(format!("/proc/{pid}/net/tcp"))?;
    let inodes: HashSet<_> = table
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            (fields.len() > 9 && fields[1] == local && fields[2] == remote && fields[3] == "01").then(|| format!("socket:[{}]", fields[9]))
        })
        .collect();
    if inodes.is_empty() {
        return Ok(false);
    }
    for fd in fs::read_dir(format!("/proc/{pid}/fd"))? {
        if let Ok(path) = fs::read_link(fd?.path())
            && inodes.contains(path.to_string_lossy().as_ref())
        {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(windows)]
fn platform_owns(stream: &TcpStream, pid: u32) -> io::Result<bool> {
    use std::net::SocketAddr;
    use windows_sys::Win32::{
        Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR},
        NetworkManagement::IpHelper::{GetExtendedTcpTable, MIB_TCPROW_OWNER_PID, TCP_TABLE_OWNER_PID_ALL},
        Networking::WinSock::AF_INET,
    };
    let (SocketAddr::V4(client), SocketAddr::V4(server)) = (stream.peer_addr()?, stream.local_addr()?) else {
        return Err(io::ErrorKind::Unsupported.into());
    };
    // DWORD-aligned allocation. Retry boundedly if sockets changed between
    // sizing and fetching. Inspect both endpoints, not just a reusable port.
    let mut size = 0u32;
    for _ in 0..3 {
        let mut storage = vec![0u32; (size as usize).div_ceil(4).max(1)];
        // SAFETY: storage is aligned and contains at least size writable bytes.
        let result = unsafe {
            GetExtendedTcpTable(
                storage.as_mut_ptr().cast(),
                &mut size,
                0,
                AF_INET as u32,
                TCP_TABLE_OWNER_PID_ALL,
                0,
            )
        };
        if result == ERROR_INSUFFICIENT_BUFFER {
            continue;
        }
        if result != NO_ERROR {
            return Err(io::Error::from_raw_os_error(result as i32));
        }
        let count = storage[0] as usize;
        let row_size = std::mem::size_of::<MIB_TCPROW_OWNER_PID>();
        if count > (storage.len() * 4 - 4) / row_size {
            return Err(io::ErrorKind::InvalidData.into());
        }
        for index in 0..count {
            // SAFETY: count was checked against allocation; Windows C rows are
            // DWORD-aligned and have the documented fixed layout.
            let row = unsafe { &*storage.as_ptr().add(1).cast::<MIB_TCPROW_OWNER_PID>().add(index) };
            if row.dwOwningPid == pid
                && row.dwLocalAddr == u32::from_ne_bytes(client.ip().octets())
                && row.dwRemoteAddr == u32::from_ne_bytes(server.ip().octets())
                && u16::from_be(row.dwLocalPort as u16) == client.port()
                && u16::from_be(row.dwRemotePort as u16) == server.port()
            {
                return Ok(true);
            }
        }
        return Ok(false);
    }
    Err(io::ErrorKind::WouldBlock.into())
}

#[cfg(not(any(windows, target_os = "linux")))]
fn platform_owns(_: &TcpStream, _: u32) -> io::Result<bool> {
    // No insecure fallback on a new platform.
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Media peer authorization requires Windows or Linux",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    #[cfg(any(windows, target_os = "linux"))]
    fn amante_security_exact_socket_owner_not_another_pid() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let _client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server, _) = listener.accept().unwrap();
        assert!(owns_connection(&server, std::process::id()).unwrap());
        assert!(!owns_connection(&server, 0).unwrap_or(false));
    }
}
