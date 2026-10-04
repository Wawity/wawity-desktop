use std::io::Read;
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const DOWN_ENDPOINT: &str = "https://speed.cloudflare.com/__down?bytes=";
const UP_ENDPOINT: &str = "https://speed.cloudflare.com/__up";
const META_ENDPOINT: &str = "https://speed.cloudflare.com/meta";
const IPV4_ENDPOINT: &str = "https://api.ipify.org?format=json";
const IPV6_ENDPOINT: &str = "https://api6.ipify.org?format=json";
const LEAK_ENDPOINT: &str = "https://bash.ws/dnsleak/test/";

const LANES: usize = 4;
const CHUNK: usize = 64 * 1024;
const DOWN_CHUNK_BYTES: u64 = 25_000_000;
const UP_CHUNK_BYTES: usize = 2_000_000;
const DOWN_WINDOW: Duration = Duration::from_secs(9);
const UP_WINDOW: Duration = Duration::from_secs(7);
const SAMPLE_EVERY: Duration = Duration::from_millis(220);
const RAMP: Duration = Duration::from_millis(1500);
const LATENCY_ROUNDS: usize = 9;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedTick {
    pub phase: String,
    pub mbps: f64,
    pub progress: f64,
    pub transferred: u64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeedResult {
    pub download_mbps: f64,
    pub upload_mbps: f64,
    pub ping_ms: f64,
    pub jitter_ms: f64,
    pub loss: f64,
    pub colo: String,
    pub exit_ip: String,
    pub carrier: String,
    pub country: String,
    pub down_bytes: u64,
    pub up_bytes: u64,
    pub took_ms: u64,
    pub aborted: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolverHop {
    pub ip: String,
    pub country: String,
    pub carrier: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeakAudit {
    pub exit_ip: String,
    pub exit_country: String,
    pub carrier: String,
    pub colo: String,
    pub ipv6: String,
    pub ipv6_exposed: bool,
    pub resolvers: Vec<ResolverHop>,
    pub resolver_countries: Vec<String>,
    pub dns_outside_tunnel: bool,
    pub resolver_count: usize,
    pub took_ms: u64,
}

#[derive(Deserialize, Default)]
struct EdgeMeta {
    #[serde(default)]
    colo: String,
    #[serde(default, rename = "clientIp")]
    client_ip: String,
    #[serde(default, rename = "asOrganization")]
    carrier: String,
    #[serde(default)]
    country: String,
}

#[derive(Deserialize)]
struct IpEcho {
    #[serde(default)]
    ip: String,
}

#[derive(Deserialize)]
struct LeakRow {
    #[serde(default)]
    ip: String,
    #[serde(default)]
    country_name: String,
    #[serde(default)]
    asn: String,
    #[serde(default, rename = "type")]
    kind: String,
}

fn agent(wait: Duration) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(wait)
        .connect_timeout(Duration::from_secs(6))
        .pool_max_idle_per_host(LANES)
        .user_agent("Wawity/1.0")
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

pub fn mbps(bytes: u64, span: Duration) -> f64 {
    let secs = span.as_secs_f64();
    if secs <= 0.0 || bytes == 0 {
        return 0.0;
    }
    (bytes as f64 * 8.0) / secs / 1_000_000.0
}

fn noise() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9e37_79b9)
}

pub fn rand_tag() -> String {
    let mut state = noise() ^ 0x2545_f491_4f6c_dd1d;
    let mut out = String::with_capacity(16);
    for _ in 0..4 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        out.push_str(&format!("{:04x}", (state & 0xffff) as u16));
    }
    out
}

fn first_addr(host: &str, port: u16) -> Option<SocketAddr> {
    (host, port).to_socket_addrs().ok()?.next()
}

pub fn tcp_touch(host: &str, port: u16, wait: Duration) -> Option<u64> {
    let addr = first_addr(host, port)?;
    let began = Instant::now();
    let sock = TcpStream::connect_timeout(&addr, wait).ok()?;
    let took = began.elapsed().as_micros() as u64;
    let _ = sock.shutdown(Shutdown::Both);
    Some(took)
}

pub fn tls_touch(host: &str, port: u16, sni: &str, wait: Duration) -> Option<u64> {
    use std::io::Write;

    let addr = first_addr(host, port)?;
    let began = Instant::now();
    let mut sock = TcpStream::connect_timeout(&addr, wait).ok()?;
    let _ = sock.set_nodelay(true);
    sock.set_read_timeout(Some(wait)).ok()?;
    sock.set_write_timeout(Some(wait)).ok()?;

    let hello = crate::blockcheck::synthetic_hello(sni, noise());
    if sock.write_all(&hello).is_err() {
        let _ = sock.shutdown(Shutdown::Both);
        return None;
    }
    let _ = sock.flush();

    let mut head = [0u8; 5];
    let seen = sock.read(&mut head);
    let took = began.elapsed().as_micros() as u64;
    let _ = sock.shutdown(Shutdown::Both);

    match seen {
        Ok(n) if n > 0 && (head[0] == 0x16 || head[0] == 0x15) => Some(took),
        _ => None,
    }
}

pub fn latency_burst(rounds: usize, halt: &AtomicBool) -> (f64, f64, f64) {
    let client = agent(Duration::from_secs(6));
    let url = format!("{}0", DOWN_ENDPOINT);
    let mut marks: Vec<f64> = Vec::with_capacity(rounds);
    let mut misses = 0usize;

    for _ in 0..rounds {
        if halt.load(Ordering::Relaxed) {
            break;
        }
        let began = Instant::now();
        match client.get(&url).send() {
            Ok(resp) => {
                let _ = resp.bytes();
                marks.push(began.elapsed().as_secs_f64() * 1000.0);
            }
            Err(_) => misses += 1,
        }
    }

    if marks.is_empty() {
        return (0.0, 0.0, 1.0);
    }

    marks.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let keep = marks.len().saturating_sub(1).max(1);
    let trimmed = &marks[..keep];
    let ping = trimmed.iter().sum::<f64>() / trimmed.len() as f64;

    let mut swings = 0.0;
    for pair in marks.windows(2) {
        swings += (pair[1] - pair[0]).abs();
    }
    let jitter = if marks.len() > 1 {
        swings / (marks.len() - 1) as f64
    } else {
        0.0
    };

    let loss = misses as f64 / rounds.max(1) as f64;
    (ping, jitter, loss)
}

fn pull_lane(client: &reqwest::blocking::Client, moved: &AtomicU64, halt: &AtomicBool, until: Instant) {
    let url = format!("{}{}", DOWN_ENDPOINT, DOWN_CHUNK_BYTES);
    let mut pail = vec![0u8; CHUNK];

    while Instant::now() < until && !halt.load(Ordering::Relaxed) {
        let mut body = match client.get(&url).send() {
            Ok(resp) => resp,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(120));
                continue;
            }
        };
        loop {
            if Instant::now() >= until || halt.load(Ordering::Relaxed) {
                return;
            }
            match body.read(&mut pail) {
                Ok(0) => break,
                Ok(n) => {
                    moved.fetch_add(n as u64, Ordering::Relaxed);
                }
                Err(_) => return,
            }
        }
    }
}

fn push_lane(
    client: &reqwest::blocking::Client,
    payload: &[u8],
    moved: &AtomicU64,
    halt: &AtomicBool,
    until: Instant,
) {
    while Instant::now() < until && !halt.load(Ordering::Relaxed) {
        let sent = client
            .post(UP_ENDPOINT)
            .header("content-type", "application/octet-stream")
            .body(payload.to_vec())
            .send();
        match sent {
            Ok(resp) => {
                let _ = resp.bytes();
                moved.fetch_add(payload.len() as u64, Ordering::Relaxed);
            }
            Err(_) => std::thread::sleep(Duration::from_millis(150)),
        }
    }
}

fn watch<F>(
    phase: &str,
    moved: &AtomicU64,
    halt: &AtomicBool,
    window: Duration,
    base: Instant,
    span: (f64, f64),
    tick: &mut F,
) -> (u64, f64)
where
    F: FnMut(SpeedTick),
{
    let began = Instant::now();
    let until = began + window;
    let mut ramp_bytes = 0u64;
    let mut ramp_at = began;
    let mut ramp_done = false;
    let mut last_bytes = 0u64;
    let mut last_at = began;

    while Instant::now() < until {
        if halt.load(Ordering::Relaxed) {
            break;
        }
        std::thread::sleep(SAMPLE_EVERY);

        let now = Instant::now();
        let seen = moved.load(Ordering::Relaxed);
        let slice = now.duration_since(last_at);
        let live = mbps(seen.saturating_sub(last_bytes), slice);
        last_bytes = seen;
        last_at = now;

        if !ramp_done && now.duration_since(began) >= RAMP {
            ramp_bytes = seen;
            ramp_at = now;
            ramp_done = true;
        }

        let done = (now.duration_since(began).as_secs_f64() / window.as_secs_f64()).min(1.0);
        tick(SpeedTick {
            phase: phase.to_string(),
            mbps: live,
            progress: span.0 + (span.1 - span.0) * done,
            transferred: seen,
            elapsed_ms: now.duration_since(base).as_millis() as u64,
        });
    }

    let total = moved.load(Ordering::Relaxed);
    let steady = if ramp_done {
        mbps(total.saturating_sub(ramp_bytes), ramp_at.elapsed())
    } else {
        mbps(total, began.elapsed())
    };
    (total, steady)
}

pub fn fetch_meta() -> EdgeMetaView {
    let client = agent(Duration::from_secs(8));
    let meta = client
        .get(META_ENDPOINT)
        .send()
        .and_then(|r| r.json::<EdgeMeta>())
        .unwrap_or_default();
    EdgeMetaView {
        colo: meta.colo,
        ip: meta.client_ip,
        carrier: meta.carrier,
        country: meta.country,
    }
}

pub struct EdgeMetaView {
    pub colo: String,
    pub ip: String,
    pub carrier: String,
    pub country: String,
}

pub fn run_speed_test<F>(halt: Arc<AtomicBool>, mut tick: F) -> SpeedResult
where
    F: FnMut(SpeedTick),
{
    let base = Instant::now();
    let mut out = SpeedResult::default();

    tick(SpeedTick {
        phase: "meta".to_string(),
        mbps: 0.0,
        progress: 0.02,
        transferred: 0,
        elapsed_ms: 0,
    });

    let meta = fetch_meta();
    out.colo = meta.colo;
    out.exit_ip = meta.ip;
    out.carrier = meta.carrier;
    out.country = meta.country;

    tick(SpeedTick {
        phase: "latency".to_string(),
        mbps: 0.0,
        progress: 0.06,
        transferred: 0,
        elapsed_ms: base.elapsed().as_millis() as u64,
    });

    let (ping, jitter, loss) = latency_burst(LATENCY_ROUNDS, &halt);
    out.ping_ms = (ping * 10.0).round() / 10.0;
    out.jitter_ms = (jitter * 10.0).round() / 10.0;
    out.loss = (loss * 1000.0).round() / 1000.0;

    if halt.load(Ordering::Relaxed) {
        out.aborted = true;
        out.took_ms = base.elapsed().as_millis() as u64;
        return out;
    }

    let down_bytes = Arc::new(AtomicU64::new(0));
    let down_until = Instant::now() + DOWN_WINDOW + Duration::from_millis(400);
    let mut crew = Vec::with_capacity(LANES);
    for _ in 0..LANES {
        let sink = Arc::clone(&down_bytes);
        let stop = Arc::clone(&halt);
        crew.push(std::thread::spawn(move || {
            let client = agent(Duration::from_secs(25));
            pull_lane(&client, &sink, &stop, down_until);
        }));
    }

    let (grabbed, down_rate) = watch(
        "download",
        &down_bytes,
        &halt,
        DOWN_WINDOW,
        base,
        (0.08, 0.58),
        &mut tick,
    );
    halt_lanes(&halt, crew, &down_bytes, grabbed);
    out.down_bytes = down_bytes.load(Ordering::Relaxed);
    out.download_mbps = (down_rate * 100.0).round() / 100.0;

    if halt.load(Ordering::Relaxed) {
        out.aborted = true;
        out.took_ms = base.elapsed().as_millis() as u64;
        return out;
    }

    let filler = vec![0x5au8; UP_CHUNK_BYTES];
    let payload = Arc::new(filler);
    let up_bytes = Arc::new(AtomicU64::new(0));
    let up_until = Instant::now() + UP_WINDOW + Duration::from_millis(400);
    let mut haulers = Vec::with_capacity(LANES);
    for _ in 0..LANES {
        let sink = Arc::clone(&up_bytes);
        let stop = Arc::clone(&halt);
        let load = Arc::clone(&payload);
        haulers.push(std::thread::spawn(move || {
            let client = agent(Duration::from_secs(25));
            push_lane(&client, load.as_ref(), &sink, &stop, up_until);
        }));
    }

    let (hauled, up_rate) = watch(
        "upload",
        &up_bytes,
        &halt,
        UP_WINDOW,
        base,
        (0.60, 0.97),
        &mut tick,
    );
    halt_lanes(&halt, haulers, &up_bytes, hauled);
    out.up_bytes = up_bytes.load(Ordering::Relaxed);
    out.upload_mbps = (up_rate * 100.0).round() / 100.0;

    out.aborted = halt.load(Ordering::Relaxed);
    out.took_ms = base.elapsed().as_millis() as u64;

    tick(SpeedTick {
        phase: "done".to_string(),
        mbps: out.download_mbps,
        progress: 1.0,
        transferred: out.down_bytes + out.up_bytes,
        elapsed_ms: out.took_ms,
    });

    out
}

fn halt_lanes(
    halt: &Arc<AtomicBool>,
    crew: Vec<std::thread::JoinHandle<()>>,
    sink: &Arc<AtomicU64>,
    _seen: u64,
) {
    let ceased = halt.load(Ordering::Relaxed);
    if !ceased {
        halt.store(true, Ordering::Relaxed);
    }
    for worker in crew {
        let _ = worker.join();
    }
    if !ceased {
        halt.store(false, Ordering::Relaxed);
    }
    let _ = sink.load(Ordering::Relaxed);
}

fn stir_resolvers(tag: &str) {
    let client = agent(Duration::from_secs(3));
    let mut crew = Vec::with_capacity(6);
    for slot in 1..=6u8 {
        let probe = client.clone();
        let host = format!("https://{}.{}.bash.ws", slot, tag);
        crew.push(std::thread::spawn(move || {
            let _ = probe.get(&host).send();
        }));
    }
    for worker in crew {
        let _ = worker.join();
    }
}

pub fn country_code_of(name: &str) -> String {
    let clean = name.trim().to_lowercase();
    match clean.as_str() {
        "russia" | "russian federation" => "RU",
        "united states" | "united states of america" => "US",
        "germany" => "DE",
        "netherlands" => "NL",
        "france" => "FR",
        "united kingdom" => "GB",
        "finland" => "FI",
        "sweden" => "SE",
        "poland" => "PL",
        "turkey" | "türkiye" => "TR",
        "japan" => "JP",
        "singapore" => "SG",
        "canada" => "CA",
        "switzerland" => "CH",
        "austria" => "AT",
        "spain" => "ES",
        "italy" => "IT",
        "latvia" => "LV",
        "lithuania" => "LT",
        "estonia" => "EE",
        "kazakhstan" => "KZ",
        "ukraine" => "UA",
        "moldova" => "MD",
        "romania" => "RO",
        "czechia" | "czech republic" => "CZ",
        "hong kong" => "HK",
        "south korea" | "korea, republic of" => "KR",
        "india" => "IN",
        "brazil" => "BR",
        "australia" => "AU",
        "united arab emirates" => "AE",
        "israel" => "IL",
        "norway" => "NO",
        "denmark" => "DK",
        "belgium" => "BE",
        "ireland" => "IE",
        "portugal" => "PT",
        "bulgaria" => "BG",
        "serbia" => "RS",
        "hungary" => "HU",
        _ => "",
    }
    .to_string()
}

pub fn audit_leaks() -> LeakAudit {
    let began = Instant::now();
    let mut out = LeakAudit::default();

    let meta = fetch_meta();
    out.colo = meta.colo;
    out.exit_ip = meta.ip;
    out.carrier = meta.carrier;
    out.exit_country = meta.country;

    let client = agent(Duration::from_secs(8));

    if out.exit_ip.is_empty() {
        if let Ok(echo) = client.get(IPV4_ENDPOINT).send().and_then(|r| r.json::<IpEcho>()) {
            out.exit_ip = echo.ip;
        }
    }

    if let Ok(echo) = client.get(IPV6_ENDPOINT).send().and_then(|r| r.json::<IpEcho>()) {
        if echo.ip.contains(':') {
            out.ipv6 = echo.ip;
            out.ipv6_exposed = true;
        }
    }

    let tag = rand_tag();
    stir_resolvers(&tag);

    let verdict = format!("{}{}?json", LEAK_ENDPOINT, tag);
    if let Ok(rows) = client.get(&verdict).send().and_then(|r| r.json::<Vec<LeakRow>>()) {
        for row in rows.into_iter().filter(|r| r.kind == "dns") {
            if row.ip.is_empty() {
                continue;
            }
            let code = country_code_of(&row.country_name);
            if !code.is_empty() && !out.resolver_countries.contains(&code) {
                out.resolver_countries.push(code);
            }
            out.resolvers.push(ResolverHop {
                ip: row.ip,
                country: row.country_name,
                carrier: row.asn,
            });
        }
    }

    out.resolver_count = out.resolvers.len();

    let exit_code = country_code_of(&out.exit_country);
    let home = if exit_code.is_empty() {
        out.exit_country.to_uppercase()
    } else {
        exit_code
    };
    out.dns_outside_tunnel = !out.resolver_countries.is_empty()
        && !home.is_empty()
        && out.resolver_countries.iter().any(|c| *c != home);

    out.took_ms = began.elapsed().as_millis() as u64;
    out
}
