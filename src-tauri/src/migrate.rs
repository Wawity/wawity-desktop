
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigratedSubscription {
    pub name: String,
    pub url: Option<String>,
    #[serde(default)]
    pub inline_links: Vec<String>,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationScanResult {
    pub clients: Vec<ClientReport>,
    pub subscriptions: Vec<MigratedSubscription>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientReport {
    pub id: &'static str,
    pub name: &'static str,
    pub found: bool,
    pub detail: String,
}

fn home() -> Option<PathBuf> {
    std::env::var("USERPROFILE").ok().map(PathBuf::from)
}

fn appdata() -> Option<PathBuf> {
    std::env::var("APPDATA").ok().map(PathBuf::from)
}

fn localdata() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA").ok().map(PathBuf::from)
}

fn existing(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find(|p| p.exists()).cloned()
}

const SHARE_SCHEMES: [&str; 7] = [
    "vless://", "vmess://", "trojan://", "ss://", "ssr://", "hysteria2://", "hy2://",
];

fn looks_like_share_link(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    SHARE_SCHEMES.iter().any(|scheme| lower.starts_with(scheme))
}

fn looks_like_sub_url(line: &str) -> bool {
    let l = line.trim();
    let lower = l.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return false;
    }
    const BAD: [&str; 6] = ["github.com/", "githubusercontent.com/", "mozilla.org", "w3.org", "example.com", "localhost"];
    if BAD.iter().any(|b| lower.contains(b)) {
        return false;
    }
    l.len() > 18
}

fn harvest(text: &str) -> (Vec<String>, Vec<String>) {
    let mut links = Vec::new();
    let mut urls = Vec::new();
    for line in text.lines().chain(text.split_whitespace()) {
        let t = line.trim();
        if looks_like_share_link(t) {
            let link = t.split([' ', '"', '\'']).next().unwrap_or(t);
            if !links.iter().any(|l: &String| l == link) {
                links.push(link.to_string());
            }
        } else if looks_like_sub_url(t) {
            let url = t.split([' ', '"', '\'', ')']).next().unwrap_or(t);
            if !urls.iter().any(|u: &String| u == url) {
                urls.push(url.to_string());
            }
        }
    }
    (links, urls)
}

fn harvest_v2rayn_subitems(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for blob in text.split("{\"id\"").skip(1) {
        let remarks = blob
            .split("\"remarks\"")
            .nth(1)
            .and_then(|s| s.split('"').nth(1))
            .unwrap_or("")
            .to_string();
        let url = blob
            .split("\"url\"")
            .nth(1)
            .and_then(|s| s.split('"').nth(1))
            .unwrap_or("");
        if looks_like_sub_url(url) {
            let name = if remarks.is_empty() {
                "Imported".to_string()
            } else {
                remarks
            };
            out.push((name, url.to_string()));
        }
    }
    out
}
fn scan_v2rayn() -> (ClientReport, Vec<MigratedSubscription>) {
    let report_base = ClientReport { id: "v2rayn", name: "v2rayN", found: false, detail: String::new() };
    let Some(home) = home() else { return (report_base, vec![]) };
    let dirs = [
        home.join("Desktop\\v2rayN"),
        home.join("Downloads\\v2rayN"),
        PathBuf::from("C:\\v2rayN"),
        home.join("v2rayN"),
    ];
    let Some(dir) = existing(&dirs) else { return (report_base, vec![]) };

    let mut subs = Vec::new();

    for cfg_name in ["guiConfigs\\guiNConfig.json", "guiNConfig.json", "guiConfigs\\config.json"] {
        let cfg = dir.join(cfg_name);
        if !cfg.exists() { continue; }
        if let Ok(text) = std::fs::read_to_string(&cfg) {
            for (name, url) in harvest_v2rayn_subitems(&text) {
                subs.push(MigratedSubscription {
                    name: format!("v2rayN · {}", name),
                    url: Some(url),
                    inline_links: vec![],
                    source_path: cfg.display().to_string(),
                });
            }
        }
    }

    if subs.is_empty() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten() {
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            let is_txt = path.extension().map(|e| e == "txt").unwrap_or(false);
            if !is_txt { continue; }
            if let Ok(text) = std::fs::read_to_string(&path) {
                let (links, urls) = harvest(&text);
                for url in urls {
                    subs.push(MigratedSubscription {
                        name: "v2rayN".into(),
                        url: Some(url),
                        inline_links: vec![],
                        source_path: path.display().to_string(),
                    });
                }
                if !links.is_empty() {
                    subs.push(MigratedSubscription {
                        name: format!("v2rayN · {} nodes", links.len()),
                        url: None,
                        inline_links: links,
                        source_path: path.display().to_string(),
                    });
                }
            }
        }
    }

    let report = ClientReport {
        found: true,
        detail: dir.display().to_string(),
        ..report_base
    };
    (report, subs)
}

fn scan_clash_verge() -> (ClientReport, Vec<MigratedSubscription>) {
    let report_base = ClientReport { id: "clash-verge", name: "Clash Verge", found: false, detail: String::new() };
    let Some(appdata) = appdata() else { return (report_base, vec![]) };
    let candidates = [
        appdata.join("io.github.clash-verge-rev.clash-verge-rev"),
        appdata.join("clash-verge"),
        appdata.join("io.github.clashverge.dev"),
    ];
    let Some(dir) = existing(&candidates) else { return (report_base, vec![]) };

    let mut subs = Vec::new();

    for pf in ["profiles.yaml", "profiles.yml"] {
        let path = dir.join(pf);
        if !path.exists() { continue; }
        if let Ok(text) = std::fs::read_to_string(&path) {
            for line in text.lines() {
                let trimmed = line.trim();
                if let Some(rest) = trimmed.strip_prefix("url:") {
                    let url = rest.trim().trim_matches('"').trim_matches('\'');
                    if looks_like_sub_url(url) {
                        subs.push(MigratedSubscription {
                            name: "Clash Verge".into(),
                            url: Some(url.to_string()),
                            inline_links: vec![],
                            source_path: path.display().to_string(),
                        });
                    }
                }
            }
        }
    }

    let report = ClientReport { found: true, detail: dir.display().to_string(), ..report_base };
    (report, subs)
}

fn scan_clash_meta_core() -> (ClientReport, Vec<MigratedSubscription>) {
    let report_base = ClientReport { id: "clash-meta", name: "Clash / mihomo", found: false, detail: String::new() };
    let Some(home) = home() else { return (report_base, vec![]) };
    let candidates = [
        home.join(".config\\mihomo"),
        home.join(".config\\clash"),
    ];
    let Some(dir) = existing(&candidates) else { return (report_base, vec![]) };

    let mut subs = Vec::new();
    for entry in std::fs::read_dir(&dir).into_iter().flatten() {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        let is_yaml = path
            .extension()
            .map(|e| e == "yaml" || e == "yml")
            .unwrap_or(false);
        if !is_yaml { continue; }
        if let Ok(text) = std::fs::read_to_string(&path) {
            for line in text.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("url:") {
                    let url = rest.trim().trim_matches('"').trim_matches('\'');
                    if looks_like_sub_url(url) && !subs.iter().any(|s: &MigratedSubscription| s.url.as_deref() == Some(url)) {
                        subs.push(MigratedSubscription {
                            name: "Clash".into(),
                            url: Some(url.to_string()),
                            inline_links: vec![],
                            source_path: path.display().to_string(),
                        });
                    }
                }
            }
        }
    }

    let report = ClientReport { found: true, detail: dir.display().to_string(), ..report_base };
    (report, subs)
}

fn scan_nekoray() -> (ClientReport, Vec<MigratedSubscription>) {
    let report_base = ClientReport { id: "nekoray", name: "Nekoray", found: false, detail: String::new() };
    let Some(home) = home() else { return (report_base, vec![]) };
    let dirs = [home.join("nekoray"), home.join("Downloads\\nekoray"), PathBuf::from("C:\\nekoray")];
    let Some(dir) = existing(&dirs) else { return (report_base, vec![]) };

    let groups_path = dir.join("config\\groups\\groups.json");
    let mut subs = Vec::new();
    if groups_path.exists() {
        if let Ok(text) = std::fs::read_to_string(&groups_path) {
            
            for blob in text.split('{').skip(1) {
                if let Some(seg) = blob.split("\"url\"").nth(1) {
                    let url = seg.split('"').nth(1).unwrap_or("");
                    if looks_like_sub_url(url) {
                        let name = blob
                            .split("\"name\"")
                            .nth(1)
                            .and_then(|s| s.split('"').nth(1))
                            .unwrap_or("Nekoray");
                        subs.push(MigratedSubscription {
                            name: format!("Nekoray · {}", name),
                            url: Some(url.to_string()),
                            inline_links: vec![],
                            source_path: groups_path.display().to_string(),
                        });
                    }
                }
            }
        }
    }

    let report = ClientReport { found: true, detail: dir.display().to_string(), ..report_base };
    (report, subs)
}

fn scan_hiddify() -> (ClientReport, Vec<MigratedSubscription>) {
    let report_base = ClientReport { id: "hiddify", name: "Hiddify", found: false, detail: String::new() };
    let Some(appdata) = appdata() else { return (report_base, vec![]) };
    let dir_candidates = [appdata.join("app.hiddify.com"), appdata.join("Hiddify")];
    let Some(dir) = existing(&dir_candidates) else { return (report_base, vec![]) };

    let mut subs = Vec::new();
    fn walk(dir: &std::path::Path, depth: usize, subs: &mut Vec<MigratedSubscription>, budget: &mut usize) {
        if depth > 4 || subs.len() >= 10 || *budget == 0 { return; }
        for entry in std::fs::read_dir(dir).into_iter().flatten() {
            if *budget == 0 { return; }
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            if path.is_dir() {
                walk(&path, depth + 1, subs, budget);
            } else if path.extension().map(|e| e == "json" || e == "db" || e == "txt").unwrap_or(false) {
                *budget = budget.saturating_sub(1);
                if let Ok(text) = std::fs::read_to_string(&path) {
                    let (_, urls) = harvest(&text);
                    for url in urls {
                        if !subs.iter().any(|s: &MigratedSubscription| s.url.as_deref() == Some(url.as_str())) {
                            subs.push(MigratedSubscription {
                                name: "Hiddify".into(),
                                url: Some(url),
                                inline_links: vec![],
                                source_path: path.display().to_string(),
                            });
                        }
                    }
                }
            }
        }
    }
    let mut budget: usize = 400;
    walk(&dir, 0, &mut subs, &mut budget);

    let report = ClientReport { found: true, detail: dir.display().to_string(), ..report_base };
    (report, subs)
}

fn scan_happ() -> (ClientReport, Vec<MigratedSubscription>) {
    let report_base = ClientReport { id: "happ", name: "Happ", found: false, detail: String::new() };

    // Happ (desktop) keeps its profile database in several possible spots
    // depending on version/installer — sweep them all generically.
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Some(ad) = appdata() {
        roots.push(ad.join("happ"));
        roots.push(ad.join("Happ"));
        roots.push(ad.join("app.happ"));
        roots.push(ad.join("com.happ.client"));
    }
    if let Some(ld) = localdata() {
        roots.push(ld.join("happ"));
        roots.push(ld.join("Happ"));
        roots.push(ld.join("app.happ"));
    }
    if let Some(home) = home() {
        roots.push(home.join(".happ"));
    }
    let roots: Vec<PathBuf> = roots.into_iter().filter(|p| p.exists()).collect();
    if roots.is_empty() {
        return (report_base, vec![]);
    }

    let mut subs = Vec::new();
    for root in &roots {
        sweep_for_urls(root, "Happ", 4, 12, &mut subs);
    }

    let report = ClientReport {
        found: true,
        detail: roots[0].display().to_string(),
        ..report_base
    };
    (report, subs)
}

/// Generic fallback: deep-sweep client-named directories under the standard
/// config roots, harvesting subscription URLs from ANY file (json/yaml/txt/
/// sqlite/db — read lossily, capped in size).
fn scan_generic() -> (ClientReport, Vec<MigratedSubscription>) {
    let report_base = ClientReport { id: "generic", name: "Другие клиенты", found: false, detail: String::new() };

    const KEYWORDS: [&str; 14] = [
        "happ", "v2ray", "clash", "mihomo", "neko", "sing-box", "singbox",
        "hiddify", "streisand", "foxtrot", "husi", "karing", "loon", "phantom",
    ];

    let mut roots: Vec<PathBuf> = Vec::new();
    for base in [appdata(), localdata()].into_iter().flatten() {
        if let Ok(entries) = std::fs::read_dir(&base) {
            for e in entries.flatten() {
                let name = e.file_name().to_string_lossy().to_lowercase();
                if KEYWORDS.iter().any(|k| name.contains(k)) {
                    roots.push(e.path());
                }
            }
        }
    }
    // de-dup against dirs already handled by dedicated scanners is soft:
    // the seen_urls filter in the caller prevents duplicate subscriptions.
    if roots.is_empty() {
        return (report_base, vec![]);
    }

    let mut subs = Vec::new();
    for root in &roots {
        sweep_for_urls(root, "Generic", 5, 20, &mut subs);
    }

    if subs.is_empty() {
        return (report_base, vec![]);
    }
    let report = ClientReport {
        found: true,
        detail: roots[0].display().to_string(),
        ..report_base
    };
    (report, subs)
}

/// Walks `root` (bounded depth/count) reading every plausible file lossily
/// and harvesting subscription URLs into `subs`.
fn sweep_for_urls(
    root: &std::path::Path,
    label: &str,
    max_depth: usize,
    max_subs: usize,
    subs: &mut Vec<MigratedSubscription>,
) {
    const SKIP_EXT: [&str; 12] = [
        "exe", "dll", "pak", "bin", "dat", "png", "jpg", "jpeg", "ico", "webp", "woff2", "so",
    ];
    const SKIP_DIR: [&str; 9] = [
        "cache", "caches", "gpucache", "code cache", "logs", "log", "crashpad",
        "node_modules", "shadercache",
    ];
    const MAX_FILE: u64 = 3 * 1024 * 1024;
    const MAX_FILES: usize = 600;

    fn walk(
        dir: &std::path::Path,
        depth: usize,
        label: &str,
        max_subs: usize,
        budget: &mut usize,
        subs: &mut Vec<MigratedSubscription>,
    ) {
        if depth > 5 || subs.len() >= max_subs || *budget == 0 { return; }
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            if *budget == 0 { return; }
            let Ok(meta) = entry.metadata() else { continue };
            let path = entry.path();
            if meta.is_dir() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if SKIP_DIR.contains(&name.as_str()) { continue; }
                walk(&path, depth + 1, label, max_subs, budget, subs);
                continue;
            }
            *budget = budget.saturating_sub(1);
            if meta.len() == 0 || meta.len() > MAX_FILE { continue; }
            let ext = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if SKIP_EXT.contains(&ext.as_str()) { continue; }

            let bytes = match std::fs::read(&path) { Ok(b) => b, Err(_) => continue };
            let text = String::from_utf8_lossy(&bytes);
            let (_, urls) = harvest(&text);
            for url in urls {
                if subs.iter().any(|s: &MigratedSubscription| s.url.as_deref() == Some(url.as_str())) {
                    continue;
                }
                subs.push(MigratedSubscription {
                    name: label.to_string(),
                    url: Some(url),
                    inline_links: vec![],
                    source_path: path.display().to_string(),
                });
                if subs.len() >= max_subs { return; }
            }
        }
    }
    let mut budget: usize = MAX_FILES;
    walk(root, 0, label, max_subs, &mut budget, subs);
}

#[tauri::command]
pub async fn scan_foreign_clients() -> MigrationScanResult {
    tokio::task::spawn_blocking(scan_all).await.unwrap_or(MigrationScanResult {
        clients: Vec::new(),
        subscriptions: Vec::new(),
    })
}

fn scan_all() -> MigrationScanResult {
    let scanners: Vec<fn() -> (ClientReport, Vec<MigratedSubscription>)> = vec![
        scan_v2rayn,
        scan_clash_verge,
        scan_clash_meta_core,
        scan_nekoray,
        scan_hiddify,
        scan_happ,
        scan_generic,
    ];

    let mut clients = Vec::new();
    let mut subscriptions = Vec::new();
    let mut seen_urls: Vec<String> = Vec::new();

    for scanner in scanners {
        let (report, subs) = scanner();
        clients.push(report);
        for sub in subs {
            match &sub.url {
                Some(url) => {
                    if seen_urls.iter().any(|u| u == url) { continue; }
                    seen_urls.push(url.clone());
                }
                None => {}
            }
            subscriptions.push(sub);
        }
    }

    MigrationScanResult { clients, subscriptions }
}
