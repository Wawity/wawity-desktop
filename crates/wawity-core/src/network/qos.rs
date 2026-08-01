use std::time::Duration;

fn exe_names(paths: &[String]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for path in paths {
        let Some(raw) = path.rsplit(['\\', '/']).next() else { continue };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !names.iter().any(|known| known.eq_ignore_ascii_case(trimmed)) {
            names.push(trimmed.to_string());
        }
    }
    names
}

fn ps_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn purge_snippet() -> String {
    "Get-NetQosPolicy -ErrorAction SilentlyContinue | Where-Object { $_.Name -like 'WawityQos*' } | Remove-NetQosPolicy -Confirm:$false -ErrorAction SilentlyContinue".to_string()
}

pub fn refresh_async(bypass_paths: Vec<String>, core_path: Option<String>) {
    std::thread::spawn(move || {
        let game_names = exe_names(&bypass_paths);
        let core_names = match core_path {
            Some(ref p) => exe_names(std::slice::from_ref(&p.to_string())),
            None => Vec::new(),
        };

        let mut script = String::from("$ErrorActionPreference='SilentlyContinue'; ");
        script.push_str(&purge_snippet());

        for (idx, name) in game_names.iter().enumerate() {
            script.push_str(&format!(
                "; New-NetQosPolicy -Name 'WawityQosGame{}' -AppPathNameMatchCondition {} -DSCPAction 46 -NetworkProfile All | Out-Null",
                idx,
                ps_quote(name)
            ));
        }
        if let Some(name) = core_names.first() {
            script.push_str(&format!(
                "; New-NetQosPolicy -Name 'WawityQosTunnel' -AppPathNameMatchCondition {} -DSCPAction 8 -NetworkProfile All | Out-Null",
                ps_quote(name)
            ));
        }

        match crate::util::run_ps_script(&script, Duration::from_secs(25)) {
            Ok(_) => crate::util::net_debug_log(&format!(
                "qos refresh: games {:?} tunnel {:?}",
                game_names, core_names
            )),
            Err(e) => crate::util::net_debug_log(&format!("qos refresh failed: {}", e)),
        }
    });
}

pub fn clear_async() {
    std::thread::spawn(move || {
        let script = format!("$ErrorActionPreference='SilentlyContinue'; {}", purge_snippet());
        match crate::util::run_ps_script(&script, Duration::from_secs(25)) {
            Ok(_) => crate::util::net_debug_log("qos cleared"),
            Err(e) => crate::util::net_debug_log(&format!("qos clear failed: {}", e)),
        }
    });
}
