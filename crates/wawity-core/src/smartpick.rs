use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const CREW: usize = 12;
const ROUNDS: usize = 4;
const DIAL_WAIT: Duration = Duration::from_millis(2200);
const TLS_WAIT: Duration = Duration::from_millis(2600);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepTarget {
    pub id: String,
    pub host: String,
    pub port: u16,
    #[serde(default)]
    pub sni: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepSample {
    pub id: String,
    pub reachable: bool,
    pub connect_ms: f64,
    pub best_ms: f64,
    pub jitter_ms: f64,
    pub loss: f64,
    pub handshake_ms: f64,
    pub score: f64,
}

impl DeepSample {
    fn dead(id: String) -> Self {
        DeepSample {
            id,
            reachable: false,
            connect_ms: 0.0,
            best_ms: 0.0,
            jitter_ms: 0.0,
            loss: 1.0,
            handshake_ms: 0.0,
            score: 0.0,
        }
    }
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn endpoint(host: &str, port: u16) -> Option<SocketAddr> {
    (host, port).to_socket_addrs().ok()?.next()
}

fn dial(addr: &SocketAddr) -> Option<f64> {
    let began = Instant::now();
    let sock = TcpStream::connect_timeout(addr, DIAL_WAIT).ok()?;
    let took = began.elapsed().as_secs_f64() * 1000.0;
    let _ = sock.set_nodelay(true);
    let _ = sock.shutdown(Shutdown::Both);
    Some(took)
}

fn grade(connect: f64, jitter: f64, loss: f64, handshake: f64, alive: bool) -> f64 {
    if !alive {
        return 0.0;
    }
    let latency_hit = (connect / 4.0).min(55.0);
    let jitter_hit = (jitter / 1.5).min(20.0);
    let loss_hit = loss * 45.0;
    let tls_hit = if handshake <= 0.0 {
        14.0
    } else {
        (handshake / 14.0).min(16.0)
    };
    (100.0 - latency_hit - jitter_hit - loss_hit - tls_hit).max(1.0)
}

pub fn measure(target: &DeepTarget) -> DeepSample {
    let addr = match endpoint(&target.host, target.port) {
        Some(a) => a,
        None => return DeepSample::dead(target.id.clone()),
    };

    let mut marks: Vec<f64> = Vec::with_capacity(ROUNDS);
    let mut misses = 0usize;

    for slot in 0..ROUNDS {
        match dial(&addr) {
            Some(ms) => marks.push(ms),
            None => misses += 1,
        }
        if slot + 1 < ROUNDS {
            std::thread::sleep(Duration::from_millis(60));
        }
    }

    if marks.is_empty() {
        return DeepSample::dead(target.id.clone());
    }

    let mean = marks.iter().sum::<f64>() / marks.len() as f64;
    let best = marks.iter().cloned().fold(f64::MAX, f64::min);

    let mut swings = 0.0;
    for pair in marks.windows(2) {
        swings += (pair[1] - pair[0]).abs();
    }
    let jitter = if marks.len() > 1 {
        swings / (marks.len() - 1) as f64
    } else {
        0.0
    };

    let loss = misses as f64 / ROUNDS as f64;

    let sni = if target.sni.trim().is_empty() {
        target.host.clone()
    } else {
        target.sni.clone()
    };
    let handshake = crate::netprobe::tls_touch(&target.host, target.port, &sni, TLS_WAIT)
        .map(|micros| micros as f64 / 1000.0)
        .unwrap_or(0.0);

    DeepSample {
        id: target.id.clone(),
        reachable: true,
        connect_ms: round1(mean),
        best_ms: round1(best),
        jitter_ms: round1(jitter),
        loss: (loss * 100.0).round() / 100.0,
        handshake_ms: round1(handshake),
        score: round1(grade(mean, jitter, loss, handshake, true)),
    }
}

pub fn deep_probe(targets: Vec<DeepTarget>) -> Vec<DeepSample> {
    if targets.is_empty() {
        return Vec::new();
    }

    let workers = CREW.min(targets.len());
    let batch = (targets.len() + workers - 1) / workers;
    let mut crew = Vec::with_capacity(workers);

    for slice in targets.chunks(batch) {
        let part = slice.to_vec();
        crew.push(std::thread::spawn(move || {
            part.iter().map(measure).collect::<Vec<DeepSample>>()
        }));
    }

    let mut found = Vec::with_capacity(targets.len());
    for worker in crew {
        if let Ok(part) = worker.join() {
            found.extend(part);
        }
    }

    found.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    found
}
