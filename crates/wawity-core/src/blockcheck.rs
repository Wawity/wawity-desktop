use std::io::{Read, Write};
use std::net::{IpAddr, Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::Serialize;

const PROBE_WORKERS: usize = 16;
const DNS_TIMEOUT: Duration = Duration::from_secs(3);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(4);
const TLS_TIMEOUT: Duration = Duration::from_millis(3500);

pub const BLOCK_CANDIDATES: &[(&str, &str)] = &[
    ("youtube.com", "YouTube"),
    ("googlevideo.com", "YouTube CDN"),
    ("discord.com", "Discord"),
    ("gateway.discord.gg", "Discord Gateway"),
    ("instagram.com", "Instagram"),
    ("facebook.com", "Facebook"),
    ("x.com", "X"),
    ("twitter.com", "Twitter"),
    ("linkedin.com", "LinkedIn"),
    ("soundcloud.com", "SoundCloud"),
    ("medium.com", "Medium"),
    ("patreon.com", "Patreon"),
    ("signal.org", "Signal"),
    ("proton.me", "Proton"),
    ("speakerdeck.com", "Speaker Deck"),
    ("rutracker.org", "RuTracker"),
    ("chatgpt.com", "ChatGPT"),
    ("openai.com", "OpenAI"),
    ("claude.ai", "Claude"),
    ("spotify.com", "Spotify"),
    ("netflix.com", "Netflix"),
    ("twitch.tv", "Twitch"),
];

const SINKHOLE_V4: &[[u8; 4]] = &[
    [0, 0, 0, 0],
    [127, 0, 0, 1],
    [10, 10, 10, 10],
    [192, 168, 0, 1],
    [62, 33, 33, 33],
    [62, 33, 33, 34],
    [95, 213, 155, 202],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Reachable,
    DnsPoisoned,
    Unresolved,
    Refused,
    Unreachable,
    SniReset,
    SniTimeout,
}

impl Verdict {
    pub fn is_blocked(self) -> bool {
        !matches!(self, Verdict::Reachable | Verdict::Unresolved)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockReport {
    pub domain: String,
    pub label: String,
    pub blocked: bool,
    pub verdict: Verdict,
    pub elapsed_ms: u64,
}

fn weak_random(seed: u64) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    for chunk in out.chunks_mut(8) {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let bytes = state.to_le_bytes();
        chunk.copy_from_slice(&bytes[..chunk.len()]);
    }
    out
}

fn push_ext(buf: &mut Vec<u8>, kind: u16, body: &[u8]) {
    buf.extend_from_slice(&kind.to_be_bytes());
    buf.extend_from_slice(&(body.len() as u16).to_be_bytes());
    buf.extend_from_slice(body);
}

fn client_hello(sni: &str, seed: u64) -> Vec<u8> {
    let mut exts = Vec::with_capacity(256);

    let host = sni.as_bytes();
    let mut name = Vec::with_capacity(host.len() + 5);
    name.extend_from_slice(&((host.len() + 3) as u16).to_be_bytes());
    name.push(0);
    name.extend_from_slice(&(host.len() as u16).to_be_bytes());
    name.extend_from_slice(host);
    push_ext(&mut exts, 0x0000, &name);

    push_ext(&mut exts, 0x000b, &[0x01, 0x00]);
    push_ext(
        &mut exts,
        0x000a,
        &[0x00, 0x08, 0x00, 0x1d, 0x00, 0x17, 0x00, 0x18, 0x00, 0x19],
    );
    push_ext(&mut exts, 0x0017, &[]);
    push_ext(&mut exts, 0xff01, &[0x00]);
    push_ext(
        &mut exts,
        0x000d,
        &[
            0x00, 0x0c, 0x04, 0x03, 0x08, 0x04, 0x04, 0x01, 0x05, 0x03, 0x08, 0x05, 0x05, 0x01,
        ],
    );
    push_ext(&mut exts, 0x002b, &[0x04, 0x03, 0x04, 0x03, 0x03]);

    let mut body = Vec::with_capacity(exts.len() + 128);
    body.extend_from_slice(&[0x03, 0x03]);
    body.extend_from_slice(&weak_random(seed));
    body.push(32);
    body.extend_from_slice(&weak_random(seed.rotate_left(21)));
    let suites: [u16; 6] = [0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f, 0xc030];
    body.extend_from_slice(&((suites.len() * 2) as u16).to_be_bytes());
    for suite in suites {
        body.extend_from_slice(&suite.to_be_bytes());
    }
    body.extend_from_slice(&[0x01, 0x00]);
    body.extend_from_slice(&(exts.len() as u16).to_be_bytes());
    body.extend_from_slice(&exts);

    let mut handshake = Vec::with_capacity(body.len() + 9);
    handshake.push(0x01);
    let len = body.len();
    handshake.push((len >> 16) as u8);
    handshake.push((len >> 8) as u8);
    handshake.push(len as u8);
    handshake.extend_from_slice(&body);

    let mut record = Vec::with_capacity(handshake.len() + 5);
    record.extend_from_slice(&[0x16, 0x03, 0x01]);
    record.extend_from_slice(&(handshake.len() as u16).to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

fn is_sinkhole(addr: &SocketAddr) -> bool {
    match addr.ip() {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            if v4.is_loopback() || v4.is_unspecified() || v4.is_broadcast() {
                return true;
            }
            SINKHOLE_V4.iter().any(|known| *known == octets)
        }
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified(),
    }
}

fn resolve(domain: &str) -> Result<Vec<SocketAddr>, ()> {
    let target = format!("{}:443", domain);
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let found = target.to_socket_addrs().map(|it| it.collect::<Vec<_>>());
        let _ = tx.send(found);
    });
    match rx.recv_timeout(DNS_TIMEOUT) {
        Ok(Ok(addrs)) if !addrs.is_empty() => Ok(addrs),
        _ => Err(()),
    }
}

fn inspect(domain: &str, seed: u64) -> Verdict {
    let addrs = match resolve(domain) {
        Ok(list) => list,
        Err(()) => return Verdict::Unresolved,
    };
    if addrs.iter().all(is_sinkhole) {
        return Verdict::DnsPoisoned;
    }
    let addr = match addrs.iter().find(|a| !is_sinkhole(a)) {
        Some(a) => *a,
        None => return Verdict::DnsPoisoned,
    };

    let mut stream = match TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) {
        Ok(s) => s,
        Err(e) => {
            return match e.kind() {
                std::io::ErrorKind::ConnectionRefused => Verdict::Refused,
                _ => Verdict::Unreachable,
            }
        }
    };
    let _ = stream.set_nodelay(true);
    if stream.set_read_timeout(Some(TLS_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(TLS_TIMEOUT)).is_err()
    {
        return Verdict::Unreachable;
    }

    let hello = client_hello(domain, seed);
    if let Err(e) = stream.write_all(&hello) {
        let _ = stream.shutdown(Shutdown::Both);
        return match e.kind() {
            std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted => {
                Verdict::SniReset
            }
            _ => Verdict::Unreachable,
        };
    }
    let _ = stream.flush();

    let mut head = [0u8; 5];
    let outcome = stream.read(&mut head);
    let _ = stream.shutdown(Shutdown::Both);

    match outcome {
        Ok(0) => Verdict::SniReset,
        Ok(_) if head[0] == 0x16 || head[0] == 0x15 => Verdict::Reachable,
        Ok(_) => Verdict::SniReset,
        Err(e) => match e.kind() {
            std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted => {
                Verdict::SniReset
            }
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => Verdict::SniTimeout,
            _ => Verdict::Unreachable,
        },
    }
}

pub fn probe(domains: Vec<(String, String)>) -> Vec<BlockReport> {
    if domains.is_empty() {
        return Vec::new();
    }
    let base = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x5bf0_3635);

    let workers = PROBE_WORKERS.min(domains.len());
    let batch = (domains.len() + workers - 1) / workers;
    let mut crew = Vec::with_capacity(workers);

    for (index, slice) in domains.chunks(batch).enumerate() {
        let part = slice.to_vec();
        let seed = base.wrapping_add((index as u64).wrapping_mul(0x1000_0001));
        crew.push(std::thread::spawn(move || {
            part.into_iter()
                .enumerate()
                .map(|(offset, (domain, label))| {
                    let started = std::time::Instant::now();
                    let verdict = inspect(&domain, seed.wrapping_add(offset as u64));
                    BlockReport {
                        domain,
                        label,
                        blocked: verdict.is_blocked(),
                        verdict,
                        elapsed_ms: started.elapsed().as_millis() as u64,
                    }
                })
                .collect::<Vec<BlockReport>>()
        }));
    }

    let mut found = Vec::with_capacity(domains.len());
    for worker in crew {
        if let Ok(part) = worker.join() {
            found.extend(part);
        }
    }
    found
}

pub fn probe_defaults() -> Vec<BlockReport> {
    probe(
        BLOCK_CANDIDATES
            .iter()
            .map(|(domain, label)| (domain.to_string(), label.to_string()))
            .collect(),
    )
}
