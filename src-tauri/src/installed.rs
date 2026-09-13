use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
}

#[cfg(target_os = "windows")]
const JUNK_HINTS: [&str; 10] = [
    "unins",
    "uninstall",
    "setup",
    "installer",
    "updater",
    "crashhandler",
    "crashreport",
    "repair",
    "vc_redist",
    "dxsetup",
];

#[cfg(target_os = "windows")]
fn is_junk_exe(file_name: &str) -> bool {
    JUNK_HINTS.iter().any(|hint| file_name.contains(hint))
}

#[cfg(target_os = "windows")]
fn read_u16(buf: &[u8], offset: usize) -> Option<u16> {
    let end = offset.checked_add(2)?;
    let bytes = buf.get(offset..end)?;
    Some(u16::from_le_bytes([bytes[0], bytes[1]]))
}

#[cfg(target_os = "windows")]
fn read_u32(buf: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let bytes = buf.get(offset..end)?;
    Some(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

#[cfg(target_os = "windows")]
fn ansi_string_at(buf: &[u8], offset: usize) -> Option<String> {
    let slice = buf.get(offset..)?;
    let len = slice.iter().position(|b| *b == 0)?;
    if len == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&slice[..len]).to_string())
}

#[cfg(target_os = "windows")]
fn wide_string_at(buf: &[u8], offset: usize, max_bytes: usize) -> Option<String> {
    let slice = buf.get(offset..)?;
    let limit = max_bytes.min(slice.len());
    let mut units = Vec::new();
    let mut cursor = 0;
    while cursor + 1 < limit {
        let unit = u16::from_le_bytes([slice[cursor], slice[cursor + 1]]);
        if unit == 0 {
            break;
        }
        units.push(unit);
        cursor += 2;
    }
    if units.is_empty() {
        return None;
    }
    Some(String::from_utf16_lossy(&units))
}

#[cfg(target_os = "windows")]
fn expand_env_tokens(raw: &str) -> String {
    let mut result = String::new();
    let mut rest = raw;
    while let Some(start) = rest.find('%') {
        result.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find('%') {
            let var_name = &after[..end];
            let looked = std::env::vars()
                .find(|(key, _)| key.eq_ignore_ascii_case(var_name))
                .map(|(_, value)| value);
            match looked {
                Some(value) => result.push_str(&value),
                None => {
                    result.push('%');
                    result.push_str(var_name);
                    result.push('%');
                }
            }
            rest = &after[end + 1..];
        } else {
            result.push('%');
            rest = after;
            break;
        }
    }
    result.push_str(rest);
    result
}

#[cfg(target_os = "windows")]
fn shortcut_candidates(buf: &[u8]) -> Vec<String> {
    let mut candidates = Vec::new();
    if buf.len() < 0x4C || read_u32(buf, 0) != Some(0x4C) {
        return candidates;
    }
    let Some(flags) = read_u32(buf, 20) else { return candidates };
    let unicode_strings = flags & 0x80 != 0;
    let mut pos = 0x4C_usize;
    if flags & 0x01 != 0 {
        let Some(id_size) = read_u16(buf, pos) else { return candidates };
        pos += 2 + id_size as usize;
    }
    if flags & 0x02 != 0 {
        let info_start = pos;
        let Some(info_size) = read_u32(buf, info_start) else { return candidates };
        if info_size < 0x1C {
            return candidates;
        }
        let header_size = read_u32(buf, info_start + 4).unwrap_or(0);
        let info_flags = read_u32(buf, info_start + 8).unwrap_or(0);
        if info_flags & 0x01 != 0 {
            let mut local = None;
            if header_size >= 0x24 {
                if let Some(off) = read_u32(buf, info_start + 28) {
                    local = wide_string_at(buf, info_start + off as usize, 4096);
                }
            }
            if local.is_none() {
                if let Some(off) = read_u32(buf, info_start + 16) {
                    local = ansi_string_at(buf, info_start + off as usize);
                }
            }
            let mut suffix = None;
            if header_size >= 0x24 {
                if let Some(off) = read_u32(buf, info_start + 32) {
                    suffix = wide_string_at(buf, info_start + off as usize, 4096);
                }
            }
            if suffix.is_none() {
                if let Some(off) = read_u32(buf, info_start + 24) {
                    suffix = ansi_string_at(buf, info_start + off as usize);
                }
            }
            if let Some(base) = local {
                let mut full = base;
                if let Some(tail) = suffix {
                    if !tail.is_empty() {
                        if !full.ends_with('\\') {
                            full.push('\\');
                        }
                        full.push_str(&tail);
                    }
                }
                candidates.push(full);
            }
        }
        pos = info_start + info_size as usize;
    }
    let mut relative = None;
    for bit in [2u32, 3, 4, 5, 6] {
        if flags & (1 << bit) == 0 {
            continue;
        }
        let Some(count) = read_u16(buf, pos) else { return candidates };
        let byte_len = if unicode_strings {
            count as usize * 2
        } else {
            count as usize
        };
        let value_start = pos + 2;
        if bit == 3 {
            relative = if unicode_strings {
                wide_string_at(buf, value_start, byte_len)
            } else {
                buf.get(value_start..value_start + byte_len)
                    .map(|s| String::from_utf8_lossy(s).to_string())
            };
        }
        pos = value_start + byte_len;
    }
    if let Some(rel) = relative {
        candidates.push(rel);
    }
    while pos + 8 <= buf.len() {
        let Some(block_size) = read_u32(buf, pos) else { break };
        if block_size < 8 {
            break;
        }
        let Some(signature) = read_u32(buf, pos + 4) else { break };
        if signature == 0xA000_0001 && pos + 8 + 260 + 520 <= buf.len() {
            if let Some(wide) = wide_string_at(buf, pos + 8 + 260, 520) {
                candidates.push(expand_env_tokens(&wide));
            } else if let Some(narrow) = ansi_string_at(buf, pos + 8) {
                candidates.push(expand_env_tokens(&narrow));
            }
        }
        pos += block_size as usize;
    }
    candidates
}

#[cfg(target_os = "windows")]
fn resolve_shortcut(link_path: &Path) -> Option<String> {
    let meta = std::fs::metadata(link_path).ok()?;
    if meta.len() > 1_000_000 {
        return None;
    }
    let buf = std::fs::read(link_path).ok()?;
    for candidate in shortcut_candidates(&buf) {
        let cleaned = candidate.trim();
        if cleaned.is_empty() {
            continue;
        }
        let resolved = if cleaned.contains(':') || cleaned.starts_with("\\\\") {
            PathBuf::from(cleaned)
        } else {
            match link_path.parent() {
                Some(parent) => parent.join(cleaned),
                None => PathBuf::from(cleaned),
            }
        };
        let text = resolved.to_string_lossy().to_string();
        if !text.to_lowercase().ends_with(".exe") {
            continue;
        }
        if resolved.exists() {
            return Some(text);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn collect_shortcuts(dir: &Path, depth_left: usize, out: &mut Vec<InstalledApp>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if depth_left > 0 {
                collect_shortcuts(&path, depth_left - 1, out);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("lnk"))
            .unwrap_or(false)
        {
            if let Some(target) = resolve_shortcut(&path) {
                let label = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("")
                    .to_string();
                out.push(InstalledApp {
                    name: label,
                    path: target,
                });
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn start_menu_apps() -> Vec<InstalledApp> {
    let mut found = Vec::new();
    let mut roots = Vec::new();
    if let Ok(program_data) = std::env::var("ProgramData") {
        roots.push(PathBuf::from(program_data).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    if let Ok(app_data) = std::env::var("APPDATA") {
        roots.push(PathBuf::from(app_data).join("Microsoft\\Windows\\Start Menu\\Programs"));
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        roots.push(PathBuf::from(profile).join("Desktop"));
    }
    if let Ok(public) = std::env::var("PUBLIC") {
        roots.push(PathBuf::from(public).join("Desktop"));
    }
    for root in roots {
        collect_shortcuts(&root, 6, &mut found);
    }
    found
}

#[cfg(target_os = "windows")]
fn registry_line_value(line: &str, key: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with(key) {
        return None;
    }
    let rest = trimmed[key.len()..].trim_start();
    if !rest.starts_with("REG_") {
        return None;
    }
    let gap = rest.find(char::is_whitespace)?;
    let value = rest[gap..].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(target_os = "windows")]
fn icon_to_exe(raw: &str) -> Option<String> {
    let mut candidate = raw.trim().trim_matches('"').to_string();
    if let Some(comma) = candidate.rfind(',') {
        let tail = candidate[comma + 1..].trim();
        if !tail.is_empty() && tail.trim_start_matches('-').chars().all(|c| c.is_ascii_digit()) {
            candidate = candidate[..comma].trim().trim_matches('"').to_string();
        }
    }
    if !candidate.to_lowercase().ends_with(".exe") {
        return None;
    }
    if !Path::new(&candidate).exists() {
        return None;
    }
    Some(candidate)
}

#[cfg(target_os = "windows")]
fn exe_from_location(raw: &str, label: &str) -> Option<String> {
    let cleaned = raw.trim().trim_matches('"');
    if cleaned.is_empty() {
        return None;
    }
    let dir = PathBuf::from(cleaned);
    if !dir.is_dir() {
        return None;
    }
    let label_lower = label.to_lowercase();
    let first_word = label_lower.split_whitespace().next().unwrap_or("");
    let mut best_score = 0u64;
    let mut best_path = None;
    let Ok(entries) = std::fs::read_dir(&dir) else { return None };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(value) => value.to_lowercase(),
            None => continue,
        };
        if !name.ends_with(".exe") || is_junk_exe(&name) {
            continue;
        }
        let size = entry.metadata().map(|meta| meta.len()).unwrap_or(0);
        let mut score = size.max(1);
        if first_word.len() > 2 && name.contains(first_word) {
            score += 1_000_000_000_000;
        }
        if score > best_score {
            best_score = score;
            best_path = Some(path.to_string_lossy().to_string());
        }
    }
    best_path
}

#[cfg(target_os = "windows")]
fn registry_apps() -> Vec<InstalledApp> {
    let roots = [
        "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        "HKLM\\SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
    ];
    let mut found = Vec::new();
    for root in roots {
        let mut probe = crate::util::silent_command("reg");
        probe.args(&["query", root, "/s"]);
        let output = match probe.output() {
            Ok(out) if out.status.success() => out,
            _ => continue,
        };
        let text = String::from_utf8_lossy(&output.stdout);
        let mut label = String::new();
        let mut icon = String::new();
        let mut location = String::new();
        for line in text.lines().chain(std::iter::once("HKEY_END")) {
            if line.starts_with("HKEY_") {
                if !label.is_empty() {
                    let exe = icon_to_exe(&icon)
                        .or_else(|| exe_from_location(&location, &label));
                    if let Some(exe) = exe {
                        found.push(InstalledApp {
                            name: label.clone(),
                            path: exe,
                        });
                    }
                }
                label.clear();
                icon.clear();
                location.clear();
                continue;
            }
            if let Some(value) = registry_line_value(line, "DisplayName") {
                label = value;
            } else if let Some(value) = registry_line_value(line, "DisplayIcon") {
                icon = value;
            } else if let Some(value) = registry_line_value(line, "InstallLocation") {
                location = value;
            }
        }
    }
    found
}

#[cfg(target_os = "windows")]
fn gather_installed_apps() -> Vec<InstalledApp> {
    let mut taken: HashSet<String> = HashSet::new();
    let mut apps: Vec<InstalledApp> = Vec::new();
    for entry in start_menu_apps().into_iter().chain(registry_apps()) {
        let normalized = crate::util::true_case_path(&entry.path);
        if normalized.is_empty() {
            continue;
        }
        let lower = normalized.to_lowercase();
        if lower.contains("\\windows\\") {
            continue;
        }
        let file_name = lower.rsplit('\\').next().unwrap_or("");
        if is_junk_exe(file_name) {
            continue;
        }
        if !taken.insert(lower) {
            continue;
        }
        let name = if entry.name.trim().is_empty() {
            Path::new(&normalized)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(&normalized)
                .to_string()
        } else {
            entry.name.trim().to_string()
        };
        apps.push(InstalledApp {
            name,
            path: normalized,
        });
    }
    apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    apps
}

#[tauri::command]
pub async fn list_installed_apps() -> Result<Vec<InstalledApp>, String> {
    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "windows")]
        {
            Ok(gather_installed_apps())
        }
        #[cfg(not(target_os = "windows"))]
        Ok(vec![])
    })
    .await
    .map_err(|e| e.to_string())?
}
