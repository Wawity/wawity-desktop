use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallRule {
    pub name: String,
    pub enabled: bool,
    pub direction: String,
    pub action: String,
    pub profiles: String,
}

#[tauri::command]
pub fn firewall_wawity_rules() -> Result<Vec<FirewallRule>, String> {
    let script = "\
$rules = Get-NetFirewallRule -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like '*Wawity*' -or $_.DisplayName -like '*sing-box*' }; \
foreach ($r in $rules) { \
  $a = ($r | Get-NetFirewallAddressFilter -ErrorAction SilentlyContinue); \
  [pscustomobject]@{ name=$r.DisplayName; enabled=$r.Enabled; direction=$r.Direction; action=$r.Action; profiles=($r.Profile -join ',') } | ConvertTo-Json -Compress \
}".to_string();

    let output = crate::util::run_ps_script(&script, std::time::Duration::from_secs(12))
        .map_err(|e| format!("firewall query failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("firewall query error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut rules = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        let Ok(v) = parsed else { continue };
        rules.push(FirewallRule {
            name: v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
            enabled: v
                .get("enabled")
                .map(|x| {
                    x.as_bool().unwrap_or_else(|| {
                        x.as_str().map(|s| s.eq_ignore_ascii_case("true")).unwrap_or(false)
                    })
                })
                .unwrap_or(false),
            direction: v
                .get("direction")
                .and_then(|x| x.as_str())
                .unwrap_or("?")
                .to_string(),
            action: v.get("action").and_then(|x| x.as_str()).unwrap_or("?").to_string(),
            profiles: v
                .get("profiles")
                .and_then(|x| x.as_str())
                .unwrap_or("any")
                .to_string(),
        });
    }

    rules.sort_by(|a, b| a.name.cmp(&b.name));
    rules.dedup_by(|a, b| a.name == b.name);
    Ok(rules)
}
