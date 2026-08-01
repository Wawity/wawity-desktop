use crate::error::VpnError;
use std::path::PathBuf;

pub struct TunManager {
    interface_name: String,
}

impl TunManager {
    pub fn new(interface_name: &str) -> Self {
        Self {
            interface_name: interface_name.to_string(),
        }
    }

    pub fn setup_interface(&self) -> Result<(), VpnError> {
        self.ensure_wintun_next_to_singbox()?;
        Ok(())
    }

    pub fn teardown_interface(&self) -> Result<(), VpnError> {
        Ok(())
    }

    fn singbox_dir() -> Result<PathBuf, VpnError> {
        let exe_dir = std::env::current_exe()
            .map_err(|e| VpnError::IoError(e.to_string()))?
            .parent()
            .ok_or_else(|| VpnError::Internal("Cannot get exe directory".into()))?
            .to_path_buf();

        let candidates = [
            exe_dir.join("sing-box-x86_64.exe"),
            exe_dir.join("binaries").join("sing-box-x86_64.exe"),
            exe_dir.join("resources").join("sing-box-x86_64.exe"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                return Ok(candidate
                    .parent()
                    .ok_or_else(|| VpnError::Internal("Cannot get sing-box parent dir".into()))?
                    .to_path_buf());
            }
        }

        Ok(exe_dir)
    }

    fn ensure_wintun_next_to_singbox(&self) -> Result<(), VpnError> {
        let exe_dir = std::env::current_exe()
            .map_err(|e| VpnError::IoError(e.to_string()))?
            .parent()
            .ok_or_else(|| VpnError::Internal("Cannot get exe directory".into()))?
            .to_path_buf();

        let singbox_dir = Self::singbox_dir()?;

        let source_candidates = [
            exe_dir.join("wintun.dll"),
            exe_dir.join("binaries").join("wintun.dll"),
            exe_dir.join("resources").join("wintun.dll"),
        ];

        let mut source_path: Option<PathBuf> = None;
        for candidate in &source_candidates {
            if candidate.exists() {
                source_path = Some(candidate.clone());
                break;
            }
        }

        let Some(src) = source_path else {
            return Err(VpnError::WintunError(
                "wintun.dll not found in app directory or binaries/. Place wintun.dll next to the app.".into()
            ));
        };

        let dest = singbox_dir.join("wintun.dll");

        if !dest.exists() {
            std::fs::copy(&src, &dest).map_err(|e| {
                VpnError::WintunError(format!(
                    "Failed to copy wintun.dll to sing-box directory {}: {}",
                    dest.display(),
                    e
                ))
            })?;
        } else {
            let src_meta = std::fs::metadata(&src).ok();
            let dst_meta = std::fs::metadata(&dest).ok();
            let should_update = match (src_meta, dst_meta) {
                (Some(s), Some(d)) => s.len() != d.len(),
                _ => false,
            };
            if should_update {
                let _ = std::fs::remove_file(&dest);
                std::fs::copy(&src, &dest).map_err(|e| {
                    VpnError::WintunError(format!("Failed to update wintun.dll: {}", e))
                })?;
            }
        }

        Ok(())
    }

    pub fn get_interface_name(&self) -> &str {
        &self.interface_name
    }
}