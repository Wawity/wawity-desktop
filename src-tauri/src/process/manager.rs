use crate::error::VpnError;
use crate::util::{strip_unc_prefix, ABOVE_NORMAL_PRIORITY_CLASS, CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const OUTPUT_BUFFER_CAP: usize = 500;

pub struct ProcessManager {
    child: Arc<Mutex<Option<Child>>>,
    config_path: std::path::PathBuf,
    binary_path: std::path::PathBuf,
    output_buffer: Arc<Mutex<VecDeque<String>>>,
}

impl ProcessManager {
    pub fn new() -> Result<Self, VpnError> {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("wawity_singbox_config.json");

        let binary_path = Self::find_singbox_binary()?;

        if !binary_path.exists() {
            return Err(VpnError::Internal(format!(
                "sing-box binary not found at: {}. Reinstall the application.",
                binary_path.display()
            )));
        }

        Ok(Self {
            child: Arc::new(Mutex::new(None)),
            config_path,
            binary_path,
            output_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(OUTPUT_BUFFER_CAP))),
        })
    }

    fn find_singbox_binary() -> Result<std::path::PathBuf, VpnError> {
        let exe_dir = std::env::current_exe()
            .map_err(|e| VpnError::IoError(e.to_string()))?
            .parent()
            .ok_or_else(|| VpnError::Internal("Cannot get exe directory".into()))?
            .to_path_buf();

        let candidates = vec![
            exe_dir.join("sing-box-x86_64.exe"),
            exe_dir.join("binaries").join("sing-box-x86_64.exe"),
            exe_dir.join("resources").join("sing-box-x86_64.exe"),
            exe_dir.parent()
                .map(|p| p.join("resources").join("sing-box-x86_64.exe"))
                .unwrap_or_default(),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return match candidate.canonicalize() {
                    Ok(p) => Ok(strip_unc_prefix(p)),
                    Err(_) => Ok(candidate),
                };
            }
        }

        Err(VpnError::Internal(
            "sing-box binary not found in application directory. Place sing-box-x86_64.exe in binaries/ folder and rebuild."
                .into(),
        ))
    }

    pub fn binary_path_string(&self) -> String {
        self.binary_path.to_string_lossy().to_string()
    }

    pub fn kill_orphans(&self) {
        #[cfg(windows)]
        {
            let path = self.binary_path_string().replace('\'', "''");
            let script = format!(
                "Get-CimInstance Win32_Process | Where-Object {{ $_.ExecutablePath -eq '{}' }} | ForEach-Object  Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue ",
                path
            );
            let _ = crate::util::run_ps_script(&script, std::time::Duration::from_secs(6));
        }
    }

    fn push_line(buffer: &Arc<Mutex<VecDeque<String>>>, line: String) {
        let mut buf = buffer.lock().unwrap();
        if buf.len() >= OUTPUT_BUFFER_CAP {
            buf.pop_front();
        }
        buf.push_back(line);
    }

    pub fn recent_output(&self, max_lines: usize) -> String {
        let buf = self.output_buffer.lock().unwrap();
        buf.iter()
            .rev()
            .take(max_lines)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn start(&self, config_json: &str) -> Result<(), VpnError> {
        let mut child_lock = self.child.lock().unwrap();

        if child_lock.is_some() {
            return Err(VpnError::AlreadyRunning);
        }

        std::fs::write(&self.config_path, config_json)?;

        {
            let mut buf = self.output_buffer.lock().unwrap();
            buf.clear();
        }

        let mut cmd = Command::new(&self.binary_path);
        cmd.arg("run")
            .arg("-c")
            .arg(&self.config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW | ABOVE_NORMAL_PRIORITY_CLASS);

        let mut child = cmd
            .spawn()
            .map_err(|e| VpnError::ProcessError(format!("Failed to spawn sing-box: {}", e)))?;

        if let Some(stdout) = child.stdout.take() {
            let buffer = Arc::clone(&self.output_buffer);
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(l) => Self::push_line(&buffer, l),
                        Err(_) => break,
                    }
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            let buffer = Arc::clone(&self.output_buffer);
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(l) => Self::push_line(&buffer, l),
                        Err(_) => break,
                    }
                }
            });
        }

        *child_lock = Some(child);
        Ok(())
    }

    pub fn stop(&self) -> Result<(), VpnError> {
        let mut child_lock = self.child.lock().unwrap();

        let mut child = match child_lock.take() {
            Some(c) => c,
            None => return Err(VpnError::NotRunning),
        };

        if let Ok(Some(_)) = child.try_wait() {
            let _ = std::fs::remove_file(&self.config_path);
            return Ok(());
        }

        let pid = child.id();

        #[cfg(windows)]
        {
            unsafe {
                let _ = windows::Win32::System::Console::GenerateConsoleCtrlEvent(
                    windows::Win32::System::Console::CTRL_BREAK_EVENT,
                    pid,
                );
            }

            let deadline = Instant::now() + Duration::from_millis(1500);
            loop {
                match child.try_wait() {
                    Ok(Some(_)) => break,
                    Ok(None) => {
                        if Instant::now() >= deadline {
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(150));
                    }
                    Err(_) => break,
                }
            }
        }

        if let Ok(None) = child.try_wait() {
            child.kill().map_err(|e| {
                VpnError::ProcessError(format!("Failed to kill sing-box: {}", e))
            })?;
        }

        let _ = child.wait();
        let _ = std::fs::remove_file(&self.config_path);

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        let mut child_lock = self.child.lock().unwrap();

        if let Some(ref mut child) = *child_lock {
            match child.try_wait() {
                Ok(Some(_)) => {
                    *child_lock = None;
                    false
                }
                Ok(None) => true,
                Err(_) => false,
            }
        } else {
            false
        }
    }

    pub fn get_pid(&self) -> Option<u32> {
        let child_lock = self.child.lock().unwrap();
        child_lock.as_ref().map(|child| child.id())
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
