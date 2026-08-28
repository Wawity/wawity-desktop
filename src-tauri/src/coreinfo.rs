use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoreInfo {
    pub found: bool,
    pub version: String,
    pub sha256: String,
    pub path: String,
}

fn find_singbox_binary() -> Option<std::path::PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();

    let mut candidates = vec![
        exe_dir.join("sing-box-x86_64.exe"),
        exe_dir.join("binaries").join("sing-box-x86_64.exe"),
        exe_dir.join("resources").join("sing-box-x86_64.exe"),
    ];
    if let Some(parent) = exe_dir.parent() {
        candidates.push(parent.join("resources").join("sing-box-x86_64.exe"));
    }

    candidates.into_iter().find(|c| c.exists())
}

fn file_sha256(path: &std::path::Path) -> Option<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Some(format!("{:x}", hasher.finalize()))
}

fn detect_version(path: &std::path::Path) -> Option<String> {
    let mut command = std::process::Command::new(path);
    command.arg("version");

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }

    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next()?;
    first_line
        .split_whitespace()
        .last()
        .map(|token| token.trim_matches(|c: char| !c.is_ascii_graphic()).to_string())
        .filter(|token| !token.is_empty())
}

#[tauri::command]
pub fn core_info() -> CoreInfo {
    match find_singbox_binary() {
        Some(path) => {
            let hash = file_sha256(&path).unwrap_or_default();
            let version =
                detect_version(&path).unwrap_or_else(|| "unknown".to_string());
            CoreInfo {
                found: true,
                version,
                sha256: hash,
                path: path.to_string_lossy().to_string(),
            }
        }
        None => CoreInfo {
            found: false,
            version: String::new(),
            sha256: String::new(),
            path: String::new(),
        },
    }
}
