use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
pub const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(windows)]
pub const ABOVE_NORMAL_PRIORITY_CLASS: u32 = 0x0000_8000;
#[cfg(windows)]
pub const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

pub fn silent_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

pub fn net_debug_log(message: &str) {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = std::env::temp_dir().join("wawity_net.log");
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        use std::io::Write;
        let _ = writeln!(file, "[{}] {}", stamp, message);
    }
}

pub fn strip_unc_prefix(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy().to_string();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else if let Some(stripped) = s.strip_prefix(r"\\.\") {
        PathBuf::from(stripped)
    } else {
        p
    }
}

pub fn normalize_windows_path(path: &str) -> String {
    let p = path.trim();
    if p.is_empty() {
        return String::new();
    }
    let stripped = if let Some(s) = p.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", s)
    } else if let Some(s) = p.strip_prefix(r"\\?\") {
        s.to_string()
    } else if let Some(s) = p.strip_prefix(r"\\.\") {
        s.to_string()
    } else {
        p.to_string()
    };
    let canonical = stripped.replace('/', "\\");
    let mut chars: Vec<char> = canonical.chars().collect();
    if chars.len() >= 3 && chars[1] == ':' && chars[2] == '\\' && chars[0].is_ascii_alphabetic() {
        chars[0] = chars[0].to_ascii_uppercase();
    }
    chars.into_iter().collect()
}

pub fn true_case_path(path: &str) -> String {
    let normalized = normalize_windows_path(path);
    if normalized.is_empty() {
        return normalized;
    }
    match std::fs::canonicalize(&normalized) {
        Ok(real) => normalize_windows_path(&strip_unc_prefix(real).to_string_lossy()),
        Err(_) => normalized,
    }
}

pub fn normalize_path_list(paths: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for p in paths {
        let norm = true_case_path(p);
        if norm.is_empty() {
            continue;
        }
        let lower = norm.to_lowercase();
        if seen.insert(lower) {
            result.push(norm);
        }
    }
    result
}

pub fn get_default_physical_interface() -> Option<String> {
    crate::network::netinfo::default_physical_interface()
}

pub fn wait_for_wintun_teardown(timeout: Duration) -> bool {
    crate::network::netinfo::wait_for_wintun_teardown(timeout)
}

pub fn get_tun_interface_name(candidate: &str) -> Option<String> {
    crate::network::netinfo::tun_interface_alias(candidate)
}

pub fn run_ps_script(script: &str, timeout: Duration) -> Result<Output, String> {
    let mut cmd = silent_command("powershell");
    cmd.args(&[
        "-NoProfile",
        "-NonInteractive",
        "-WindowStyle",
        "Hidden",
        "-Command",
        script,
    ]);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("powershell spawn failed: {}", e))?;

    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();

    let stdout_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(ref mut p) = stdout_pipe {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });
    let stderr_handle = thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(ref mut p) = stderr_pipe {
            let _ = p.read_to_end(&mut buf);
        }
        buf
    });

    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_handle.join();
                    let _ = stderr_handle.join();
                    return Err(format!("powershell command timed out after {:?}", timeout));
                }
                thread::sleep(Duration::from_millis(40));
            }
            Err(e) => return Err(format!("powershell wait failed: {}", e)),
        }
    };

    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();
    Ok(Output { status, stdout, stderr })
}

#[derive(Deserialize)]
struct PsProcEntry {
    #[serde(rename = "Id")]
    id: u32,
    #[serde(rename = "ProcessName")]
    process_name: String,
    #[serde(rename = "Path")]
    path: Option<String>,
}

pub fn enumerate_processes_with_paths() -> Result<Vec<(u32, String, String)>, String> {
    let output = run_ps_script(
        "Get-Process | Where-Object { $_.Path -ne $null -and $_.Path -ne '' } | Select-Object Id,ProcessName,Path | ConvertTo-Json -Compress",
        Duration::from_secs(20),
    )?;
    let text = String::from_utf8_lossy(&output.stdout);
    let text = text.trim();
    if text.is_empty() {
        return Ok(vec![]);
    }
    let entries: Vec<PsProcEntry> = if text.starts_with('[') {
        serde_json::from_str(text).unwrap_or_default()
    } else {
        serde_json::from_str::<PsProcEntry>(text).map(|e| vec![e]).unwrap_or_default()
    };
    Ok(entries
        .into_iter()
        .filter_map(|e| e.path.map(|p| (e.id, e.process_name, normalize_windows_path(&p))))
        .collect())
}
