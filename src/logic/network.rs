use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::logic::packet::{self, Packet};

const PORT: u16 = 80;
const IP_CHECK_URL: &str = "https://api.ipify.org";
const IP_CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ23456789";
const CODE_PREFIX: &str = "TR-";

pub fn encode_room_code(ip: &str, room_code: &str) -> Result<String, String> {
    let octets: Vec<u8> = ip
        .split('.')
        .map(|s| s.parse::<u8>().map_err(|e| format!("invalid IP octet '{}': {}", s, e)))
        .collect::<Result<Vec<_>, _>>()?;
    if octets.len() != 4 {
        return Err("IP must have exactly 4 octets".into());
    }
    let code_bytes = room_code.as_bytes();
    if code_bytes.len() != 4 {
        return Err("room code must be exactly 4 characters".into());
    }

    let mut data = [0u8; 8];
    data[0..4].copy_from_slice(&octets);
    data[4..8].copy_from_slice(code_bytes);

    let mut encoded = String::with_capacity(CODE_PREFIX.len() + 13);
    encoded.push_str(CODE_PREFIX);

    let mut bits: u128 = 0;
    for &b in &data {
        bits = (bits << 8) | (b as u128);
    }
    bits <<= 1;

    for i in (0..13).rev() {
        let idx = ((bits >> (i * 5)) & 0x1F) as usize;
        encoded.push(BASE32_ALPHABET[idx] as char);
    }

    Ok(encoded)
}

pub fn decode_room_code(code: &str) -> Result<(String, String), String> {
    let code = code.trim();
    let code = code
        .strip_prefix(CODE_PREFIX)
        .ok_or_else(|| format!("code must start with '{}'", CODE_PREFIX))?;
    if code.len() != 13 {
        return Err(format!("code must be 13 characters after prefix, got {}", code.len()));
    }

    let mut bits: u128 = 0;
    for ch in code.chars() {
        let val = match ch {
            'A'..='Z' => (ch as u8) - b'A',
            '2'..='7' => (ch as u8) - b'2' + 26,
            _ => return Err(format!("invalid character '{}' in code", ch)),
        };
        bits = (bits << 5) | (val as u128);
    }
    bits >>= 1;

    let ip = format!(
        "{}.{}.{}.{}",
        (bits >> 56) & 0xFF,
        (bits >> 48) & 0xFF,
        (bits >> 40) & 0xFF,
        (bits >> 32) & 0xFF,
    );
    let room_code_str = format!(
        "{}{}{}{}",
        (bits >> 24) as u8 as char,
        (bits >> 16) as u8 as char,
        (bits >> 8) as u8 as char,
        (bits & 0xFF) as u8 as char,
    );

    Ok((ip, room_code_str))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkState {
    Idle,
    DetectingIp,
    Hosting { ip: Option<String> },
    Connecting,
    WaitingForPlayer,
    Connected,
    Error(String),
}

pub struct NetworkHandle {
    pub state: Arc<Mutex<NetworkState>>,
    pub stream: Option<Arc<Mutex<TcpStream>>>,
    pub listener: Option<TcpListener>,
}

impl NetworkHandle {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(NetworkState::Idle)),
            stream: None,
            listener: None,
        }
    }

    pub fn get_state(&self) -> NetworkState {
        self.state.lock().unwrap().clone()
    }

    pub fn set_state(&self, s: NetworkState) {
        *self.state.lock().unwrap() = s;
    }

    pub fn take_stream(&mut self) -> Option<Arc<Mutex<TcpStream>>> {
        self.stream.take()
    }

    pub fn shutdown(&mut self) {
        self.listener.take();
        self.stream.take();
        *self.state.lock().unwrap() = NetworkState::Idle;
    }
}

pub fn detect_public_ip(handle: Arc<Mutex<NetworkHandle>>) {
    thread::spawn(move || {
        handle.lock().unwrap().set_state(NetworkState::DetectingIp);
        match reqwest_ip_check() {
            Ok(ip) => {
                handle.lock().unwrap().set_state(NetworkState::Hosting { ip: Some(ip) });
            }
            Err(_) => {
                handle.lock().unwrap().set_state(NetworkState::Hosting {
                    ip: Some("?.?.?.? (detection failed)".into()),
                });
            }
        }
    });
}

fn reqwest_ip_check() -> Result<String, Box<dyn std::error::Error>> {
    use std::net::TcpStream as StdTcpStream;
    use std::io::BufRead;

    let stream = StdTcpStream::connect_timeout(
        &"api.ipify.org:443".parse::<std::net::SocketAddr>()?,
        IP_CHECK_TIMEOUT,
    )?;

    let request = format!(
        "GET / HTTP/1.1\r\nHost: api.ipify.org\r\nConnection: close\r\n\r\n"
    );
    let mut stream = stream;
    stream.write_all(request.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;

    let mut reader = std::io::BufReader::new(&stream);
    let mut body = String::new();
    let mut headers_done = false;
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        if headers_done {
            body.push_str(&line);
        }
        if line.trim().is_empty() {
            headers_done = true;
        }
    }
    let ip = body.trim().to_string();
    if ip.is_empty() || ip.contains('<') {
        Err("Invalid IP response".into())
    } else {
        Ok(ip)
    }
}

pub fn start_host(
    handle: Arc<Mutex<NetworkHandle>>,
    room_code: String,
) {
    thread::spawn(move || {
        handle.lock().unwrap().set_state(NetworkState::WaitingForPlayer);

        let listener = match std::net::TcpListener::bind(format!("0.0.0.0:{}", PORT)) {
            Ok(l) => l,
            Err(e) => {
                handle.lock().unwrap().set_state(NetworkState::Error(
                    format!("Failed to bind port {}: {}", PORT, e)
                ));
                return;
            }
        };

        listener.set_nonblocking(true).ok();
        handle.lock().unwrap().listener = Some(listener);

        loop {
            if handle.lock().unwrap().get_state() == NetworkState::Idle {
                return;
            }

            let accept_result = {
                let h = handle.lock().unwrap();
                match h.listener.as_ref() {
                    Some(l) => l.accept(),
                    None => return,
                }
            };

            match accept_result {
                Ok((mut stream, addr)) => {
                    println!("Client connected from {}", addr);

                    let mut buf = [0u8; 4];
                    if let Err(e) = stream.read_exact(&mut buf) {
                        handle.lock().unwrap().set_state(NetworkState::Error(
                            format!("Failed to read room code: {}", e)
                        ));
                        return;
                    }

                    let received_code = String::from_utf8_lossy(&buf).to_string();
                    if received_code != room_code {
                        let _ = stream.write_all(&[0x00]);
                        continue;
                    }

                    let _ = stream.write_all(&[0x01]);

                    let mut h = handle.lock().unwrap();
                    h.set_state(NetworkState::Connected);
                    h.stream = Some(Arc::new(Mutex::new(stream)));
                    return;
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    handle.lock().unwrap().set_state(NetworkState::Error(
                        format!("Connection failed: {}", e)
                    ));
                    return;
                }
            }
        }
    });
}

pub fn connect_to_host(
    handle: Arc<Mutex<NetworkHandle>>,
    ip: String,
    room_code: String,
) {
    thread::spawn(move || {
        handle.lock().unwrap().set_state(NetworkState::Connecting);

        let addr = format!("{}:{}", ip, PORT);
        match TcpStream::connect_timeout(
            &addr.parse::<std::net::SocketAddr>().unwrap_or_else(|_| {
                std::net::SocketAddr::new(
                    std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
                    PORT,
                )
            }),
            std::time::Duration::from_secs(5),
        ) {
            Ok(mut stream) => {
                let code_bytes = room_code.as_bytes();
                if code_bytes.len() != 4 {
                    handle.lock().unwrap().set_state(NetworkState::Error(
                        "Invalid room code".into()
                    ));
                    return;
                }

                if let Err(e) = stream.write_all(code_bytes) {
                    handle.lock().unwrap().set_state(NetworkState::Error(
                        format!("Failed to send room code: {}", e)
                    ));
                    return;
                }

                let mut response = [0u8; 1];
                if let Err(e) = stream.read_exact(&mut response) {
                    handle.lock().unwrap().set_state(NetworkState::Error(
                        format!("Failed to read response: {}", e)
                    ));
                    return;
                }

                if response[0] != 0x01 {
                    handle.lock().unwrap().set_state(NetworkState::Error(
                        "Room code rejected by host".into()
                    ));
                    return;
                }

                let mut h = handle.lock().unwrap();
                h.set_state(NetworkState::Connected);
                h.stream = Some(Arc::new(Mutex::new(stream)));
            }
            Err(e) => {
                let h = handle.lock().unwrap();
                h.set_state(NetworkState::Error(
                    format!("Connection to {} failed: {}", addr, e)
                ));
            }
        }
    });
}

pub fn send_packet(
    stream: &Arc<Mutex<TcpStream>>,
    packet: &Packet,
) -> io::Result<()> {
    let data = packet::encode(packet).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut s = stream.lock().unwrap();
    s.write_all(&data)?;
    s.flush()?;
    Ok(())
}

pub fn recv_packets(
    stream: &Arc<Mutex<TcpStream>>,
    buf: &mut Vec<u8>,
) -> io::Result<Vec<Packet>> {
    let mut s = stream.lock().unwrap();
    let mut temp = [0u8; 4096];
    loop {
        s.set_nonblocking(true)?;
        match s.read(&mut temp) {
            Ok(0) => return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "connection closed")),
            Ok(n) => buf.extend_from_slice(&temp[..n]),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => break,
            Err(e) => return Err(e),
        }
    }
    s.set_nonblocking(false)?;

    let (packets, remaining) = packet::decode_all(buf);
    buf.clear();
    buf.extend_from_slice(&remaining);
    Ok(packets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let test_cases = [
            ("0.0.0.0", "ABCD"),
            ("255.255.255.255", "ZZZZ"),
            ("192.168.1.1", "A3F7"),
            ("10.0.0.1", "XYZ9"),
            ("203.0.113.42", "K7M2"),
        ];

        for (ip, room_code) in test_cases {
            let encoded = encode_room_code(ip, room_code).unwrap();
            assert!(encoded.starts_with(CODE_PREFIX), "missing prefix: {}", encoded);
            assert_eq!(encoded.len(), CODE_PREFIX.len() + 13);

            let (decoded_ip, decoded_code) = decode_room_code(&encoded).unwrap();
            assert_eq!(decoded_ip, ip);
            assert_eq!(decoded_code, room_code);
        }
    }

    #[test]
    fn decode_rejects_bad_prefix() {
        let result = decode_room_code("XX-A7K2M9Q1X3FB");
        assert!(result.is_err());
    }

    #[test]
    fn decode_rejects_wrong_length() {
        let result = decode_room_code("TR-ABC");
        assert!(result.is_err());
    }

    #[test]
    fn decode_rejects_invalid_chars() {
        let result = decode_room_code("TR-0O0O0O0O0O0O0");
        assert!(result.is_err());
    }
}
