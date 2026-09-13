use crate::util::run_ps_script;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedGame {
    pub key: String,
    pub display_name: String,
    pub exe_paths: Vec<String>,
    pub recommended: bool,
    pub launcher: String,
    pub install_dir: String,
}

struct LibraryGame {
    name: String,
    dir: PathBuf,
    launcher: &'static str,
}

const EXE_NAME_BLOCKLIST: &[&str] = &[
    "unins", "uninstall", "setup", "installer", "redist", "vcredist", "directx",
    "dxsetup", "dotnet", "crashhandler", "crashreport", "crashpad", "errorreport",
    "updater", "update", "patcher", "repair", "diagnostic", "benchmark",
    "cefprocess", "subprocess", "webhelper", "helper", "activation",
    "touchup", "prereq", "oalinst", "physx", "quicksfv", "language",
];

fn is_plausible_game_exe(name_lower: &str) -> bool {
    !EXE_NAME_BLOCKLIST.iter().any(|bad| name_lower.contains(bad))
}

fn folder_affinity(exe_stem: &str, folder: &str) -> u32 {
    let a: String = exe_stem.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    let b: String = folder.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
    if a.is_empty() || b.is_empty() {
        return 0;
    }
    if a == b {
        return 100;
    }
    if b.contains(&a) || a.contains(&b) {
        return 60;
    }
    0
}

fn pick_exes(dir: &Path) -> Vec<String> {
    let folder = dir
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut known: Vec<String> = Vec::new();
    let mut scored: Vec<(u32, u64, String)> = Vec::new();

    for exe in walk_for_exes(dir, 6, 20000) {
        let Some(fname) = exe.file_name().and_then(|f| f.to_str()) else { continue };
        let lower = fname.to_lowercase();
        let path_str = crate::util::strip_unc_prefix(exe.clone()).to_string_lossy().to_string();

        if KNOWN_GAMES.iter().any(|d| d.match_names.contains(&lower.as_str())) {
            known.push(path_str);
            continue;
        }
        if !is_plausible_game_exe(&lower) {
            continue;
        }
        let size = std::fs::metadata(&exe).map(|m| m.len()).unwrap_or(0);
        if size < 262_144 {
            continue;
        }
        let stem = lower.trim_end_matches(".exe").to_string();
        let depth = exe.components().count().saturating_sub(dir.components().count()) as u32;
        let mut score = folder_affinity(&stem, &folder);
        if depth <= 1 {
            score += 25;
        }
        if lower.contains("shipping") {
            score += 40;
        }
        scored.push((score, size, path_str));
    }

    if !known.is_empty() {
        known.sort();
        known.dedup();
        return known;
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
    scored.into_iter().take(3).map(|(_, _, p)| p).collect()
}

struct GameDef {
    key: &'static str,
    display_name: &'static str,
    match_names: &'static [&'static str],
}

const KNOWN_GAMES: &[GameDef] = &[
    GameDef { key: "cs2", display_name: "Counter-Strike 2", match_names: &["cs2.exe"] },
    GameDef { key: "dota2", display_name: "Dota 2", match_names: &["dota2.exe"] },
    GameDef { key: "valorant", display_name: "VALORANT", match_names: &["valorant.exe", "valorant-win64-shipping.exe"] },
    GameDef { key: "lol", display_name: "League of Legends", match_names: &["leagueclient.exe", "leagueclientux.exe", "league of legends.exe"] },
    GameDef { key: "pubg", display_name: "PUBG: Battlegrounds", match_names: &["tslgame.exe", "pubg.exe"] },
    GameDef { key: "fortnite", display_name: "Fortnite", match_names: &["fortniteclient-win64-shipping.exe", "fortnitelauncher.exe"] },
    GameDef { key: "apex", display_name: "Apex Legends", match_names: &["r5apex.exe"] },
    GameDef { key: "gta5", display_name: "GTA V", match_names: &["gta5.exe", "gta5_enhanced.exe"] },
    GameDef { key: "rust", display_name: "Rust", match_names: &["rustclient.exe"] },
    GameDef { key: "wow", display_name: "World of Warcraft", match_names: &["wow.exe", "wowt.exe", "wow-64.exe"] },
    GameDef { key: "overwatch2", display_name: "Overwatch 2", match_names: &["overwatch.exe"] },
    GameDef { key: "cod", display_name: "Call of Duty", match_names: &["cod.exe", "modernwarfare.exe", "blackops6.exe"] },
    GameDef { key: "destiny2", display_name: "Destiny 2", match_names: &["destiny2.exe"] },
    GameDef { key: "eft", display_name: "Escape from Tarkov", match_names: &["eft.exe", "escapefromtarkov.exe"] },
    GameDef { key: "rainbow6", display_name: "Rainbow Six Siege", match_names: &["rainbowsix.exe", "rainbowsix_vulkan.exe"] },
    GameDef { key: "minecraft", display_name: "Minecraft Launcher", match_names: &["minecraftlauncher.exe"] },
    GameDef { key: "roblox", display_name: "Roblox", match_names: &["robloxplayerbeta.exe"] },
    GameDef { key: "genshin", display_name: "Genshin Impact", match_names: &["genshinimpact.exe", "yuanshen.exe"] },
    GameDef { key: "csgo", display_name: "CS:GO (Legacy)", match_names: &["csgo.exe"] },
    GameDef { key: "tf2", display_name: "Team Fortress 2", match_names: &["tf_win64.exe", "tf.exe"] },
    GameDef { key: "deadlock", display_name: "Deadlock", match_names: &["deadlock.exe", "project8.exe"] },
    GameDef { key: "warframe", display_name: "Warframe", match_names: &["warframe.x64.exe"] },
    GameDef { key: "warthunder", display_name: "War Thunder", match_names: &["aces.exe"] },
    GameDef { key: "enlisted", display_name: "Enlisted", match_names: &["enlisted.exe"] },
    GameDef { key: "crossout", display_name: "Crossout", match_names: &["crossout.exe"] },
    GameDef { key: "wot", display_name: "World of Tanks", match_names: &["worldoftanks.exe", "wotlauncher.exe"] },
    GameDef { key: "wows", display_name: "World of Warships", match_names: &["worldofwarships.exe"] },
    GameDef { key: "rocketleague", display_name: "Rocket League", match_names: &["rocketleague.exe"] },
    GameDef { key: "dbd", display_name: "Dead by Daylight", match_names: &["deadbydaylight-win64-shipping.exe"] },
    GameDef { key: "seaofthieves", display_name: "Sea of Thieves", match_names: &["sotgame.exe"] },
    GameDef { key: "halo", display_name: "Halo Infinite", match_names: &["haloinfinite.exe"] },
    GameDef { key: "thefinals", display_name: "THE FINALS", match_names: &["discovery.exe"] },
    GameDef { key: "marvelrivals", display_name: "Marvel Rivals", match_names: &["marvel-win64-shipping.exe"] },
    GameDef { key: "hunt", display_name: "Hunt: Showdown", match_names: &["huntgame.exe"] },
    GameDef { key: "dayz", display_name: "DayZ", match_names: &["dayz_x64.exe", "dayz_be.exe"] },
    GameDef { key: "arma3", display_name: "Arma 3", match_names: &["arma3_x64.exe", "arma3.exe"] },
    GameDef { key: "squad", display_name: "Squad", match_names: &["squadgame.exe"] },
    GameDef { key: "hll", display_name: "Hell Let Loose", match_names: &["hll-win64-shipping.exe"] },
    GameDef { key: "battlefield", display_name: "Battlefield", match_names: &["bf2042.exe", "bfv.exe", "bf1.exe", "bf4.exe"] },
    GameDef { key: "eafc", display_name: "EA Sports FC", match_names: &["fc25.exe", "fc24.exe", "fifa23.exe"] },
    GameDef { key: "eldenring", display_name: "Elden Ring", match_names: &["eldenring.exe"] },
    GameDef { key: "poe", display_name: "Path of Exile", match_names: &["pathofexile_x64.exe", "pathofexile_x64steam.exe", "pathofexile.exe", "pathofexilesteam.exe"] },
    GameDef { key: "diablo4", display_name: "Diablo IV", match_names: &["diablo iv.exe"] },
    GameDef { key: "hearthstone", display_name: "Hearthstone", match_names: &["hearthstone.exe"] },
    GameDef { key: "sc2", display_name: "StarCraft II", match_names: &["sc2_x64.exe", "sc2.exe"] },
    GameDef { key: "lostark", display_name: "Lost Ark", match_names: &["lostark.exe"] },
    GameDef { key: "newworld", display_name: "New World", match_names: &["newworld.exe"] },
    GameDef { key: "albion", display_name: "Albion Online", match_names: &["albion-online.exe"] },
    GameDef { key: "smite", display_name: "SMITE", match_names: &["smite.exe"] },
    GameDef { key: "paladins", display_name: "Paladins", match_names: &["paladins.exe"] },
    GameDef { key: "brawlhalla", display_name: "Brawlhalla", match_names: &["brawlhalla.exe"] },
    GameDef { key: "amongus", display_name: "Among Us", match_names: &["among us.exe"] },
    GameDef { key: "phasmophobia", display_name: "Phasmophobia", match_names: &["phasmophobia.exe"] },
    GameDef { key: "gmod", display_name: "Garry's Mod", match_names: &["gmod.exe"] },
    GameDef { key: "l4d2", display_name: "Left 4 Dead 2", match_names: &["left4dead2.exe"] },
    GameDef { key: "payday", display_name: "PAYDAY", match_names: &["payday2_win32_release.exe", "payday3client-win64-shipping.exe"] },
    GameDef { key: "forza", display_name: "Forza Horizon 5", match_names: &["forzahorizon5.exe"] },
    GameDef { key: "naraka", display_name: "Naraka: Bladepoint", match_names: &["narakabladepoint.exe"] },
    GameDef { key: "rdr2", display_name: "Red Dead Redemption 2", match_names: &["rdr2.exe"] },
    GameDef { key: "fivem", display_name: "FiveM (GTA RP)", match_names: &["fivem.exe"] },
    GameDef { key: "ark", display_name: "ARK: Survival", match_names: &["arkascended.exe", "shootergame.exe"] },
    GameDef { key: "7dtd", display_name: "7 Days to Die", match_names: &["7daystodie.exe"] },
    GameDef { key: "terraria", display_name: "Terraria", match_names: &["terraria.exe"] },
    GameDef { key: "palworld", display_name: "Palworld", match_names: &["palworld-win64-shipping.exe"] },
    GameDef { key: "helldivers2", display_name: "Helldivers 2", match_names: &["helldivers2.exe"] },
    GameDef { key: "bg3", display_name: "Baldur's Gate 3", match_names: &["bg3.exe", "bg3_dx11.exe"] },
    GameDef { key: "cyberpunk", display_name: "Cyberpunk 2077", match_names: &["cyberpunk2077.exe"] },
    GameDef { key: "hsr", display_name: "Honkai: Star Rail", match_names: &["starrail.exe"] },
    GameDef { key: "zzz", display_name: "Zenless Zone Zero", match_names: &["zenlesszonezero.exe"] },
    GameDef { key: "wuwa", display_name: "Wuthering Waves", match_names: &["wuthering waves.exe"] },
    GameDef { key: "valheim", display_name: "Valheim", match_names: &["valheim.exe"] },
    GameDef { key: "drg", display_name: "Deep Rock Galactic", match_names: &["fsd-win64-shipping.exe"] },
    GameDef { key: "readyornot", display_name: "Ready or Not", match_names: &["readyornot-win64-shipping.exe"] },
    GameDef { key: "titanfall2", display_name: "Titanfall 2", match_names: &["titanfall2.exe"] },
    GameDef { key: "battlebit", display_name: "BattleBit Remastered", match_names: &["battlebit.exe"] },
    GameDef { key: "starcitizen", display_name: "Star Citizen", match_names: &["starcitizen.exe"] },
    GameDef { key: "division2", display_name: "The Division 2", match_names: &["thedivision2.exe"] },
    GameDef { key: "forhonor", display_name: "For Honor", match_names: &["forhonor.exe"] },
    GameDef { key: "sf6", display_name: "Street Fighter 6", match_names: &["streetfighter6.exe"] },
    GameDef { key: "tekken8", display_name: "TEKKEN 8", match_names: &["polaris-win64-shipping.exe", "tekken 8.exe"] },
    GameDef { key: "trackmania", display_name: "Trackmania", match_names: &["trackmania.exe"] },
    GameDef { key: "osu", display_name: "osu!", match_names: &["osu!.exe", "osu!.lazer.exe"] },
    GameDef { key: "mcbedrock", display_name: "Minecraft Bedrock", match_names: &["minecraft.windows.exe"] },
];

const HELPER_PROCESS_NAMES: &[&str] = &[
    "steam.exe", "steamwebhelper.exe", "steamservice.exe", "steamerrorreporter.exe", "gameoverlayui.exe",
    "epicgameslauncher.exe", "epicwebhelper.exe", "epiconlineservices.exe",
    "easyanticheat.exe", "easyanticheat_eos.exe",
    "beservice.exe", "battleye.exe",
    "riotclientservices.exe", "vgc.exe", "vgtray.exe", "vanguard.exe",
    "battle.net.exe", "agent.exe", "blizzardbrowser.exe", "blizzarderrorhandler.exe",
];

fn walk_for_exes(dir: &Path, max_depth: usize, max_files: usize) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut budget = max_files;
    walk_inner(dir, max_depth, &mut budget, &mut result);
    result
}

fn walk_inner(dir: &Path, depth_left: usize, budget: &mut usize, out: &mut Vec<PathBuf>) {
    if *budget == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        if *budget == 0 {
            return;
        }
        let path = entry.path();
        *budget -= 1;
        if path.is_dir() {
            if depth_left > 0 {
                walk_inner(&path, depth_left - 1, budget, out);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("exe"))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
}

fn get_steam_root() -> Option<String> {
    let mut probe = crate::util::silent_command("reg");
    probe.args(&["query", "HKCU\\Software\\Valve\\Steam", "/v", "SteamPath"]);
    if let Ok(output) = probe.output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if let Some(idx) = line.find("REG_SZ") {
                    let value = line[idx + 6..].trim();
                    if !value.is_empty() {
                        return Some(value.replace('/', "\\"));
                    }
                }
            }
        }
    }

    let script = "(Get-ItemProperty -Path 'HKCU:\\Software\\Valve\\Steam' -Name 'SteamPath' -ErrorAction SilentlyContinue).SteamPath";
    if let Ok(output) = run_ps_script(script, Duration::from_secs(12)) {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return Some(text.replace('/', "\\"));
            }
        }
    }

    for candidate in ["C:\\Program Files (x86)\\Steam", "C:\\Program Files\\Steam"] {
        if Path::new(candidate).join("steamapps").exists() {
            return Some(candidate.to_string());
        }
    }

    None
}

fn parse_vdf_paths(content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(idx) = trimmed.find("\"path\"") {
            let after = &trimmed[idx + 6..];
            if let Some(start) = after.find('"') {
                let rest = &after[start + 1..];
                if let Some(end) = rest.find('"') {
                    let raw = &rest[..end];
                    paths.push(raw.replace("\\\\", "\\"));
                }
            }
        }
    }
    paths
}

fn get_steam_library_paths() -> Vec<String> {
    let mut result = Vec::new();
    let Some(root) = get_steam_root() else { return result };
    result.push(root.clone());

    let vdf_path = PathBuf::from(&root).join("steamapps").join("libraryfolders.vdf");
    if let Ok(content) = std::fs::read_to_string(&vdf_path) {
        for p in parse_vdf_paths(&content) {
            let normalized = p.replace('/', "\\");
            if !result.iter().any(|r: &String| r.eq_ignore_ascii_case(&normalized)) {
                result.push(normalized);
            }
        }
    }

    result
}

fn scan_steam(found: &mut HashMap<&'static str, HashSet<String>>, helpers: &mut HashSet<String>) {
    for lib in get_steam_library_paths() {
        let common = PathBuf::from(&lib).join("steamapps").join("common");
        let Ok(entries) = std::fs::read_dir(&common) else { continue };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let exe_files = walk_for_exes(&path, 8, 30000);
            let mut folder_game_key: Option<&'static str> = None;

            for exe in &exe_files {
                let fname = exe.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();
                if let Some(def) = KNOWN_GAMES.iter().find(|d| d.match_names.contains(&fname.as_str())) {
                    folder_game_key = Some(def.key);
                }
            }

            for exe in &exe_files {
                let fname = exe.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();
                let path_str = crate::util::strip_unc_prefix(exe.clone()).to_string_lossy().to_string();

                if let Some(def) = KNOWN_GAMES.iter().find(|d| d.match_names.contains(&fname.as_str())) {
                    found.entry(def.key).or_default().insert(path_str);
                } else if HELPER_PROCESS_NAMES.contains(&fname.as_str()) {
                    if let Some(key) = folder_game_key {
                        found.entry(key).or_default().insert(path_str);
                    } else {
                        helpers.insert(path_str);
                    }
                }
            }
        }
    }

    if let Some(steam_root) = get_steam_root() {
        let root_exes = walk_for_exes(&PathBuf::from(&steam_root), 3, 6000);
        for exe in root_exes {
            let fname = exe.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();
            if HELPER_PROCESS_NAMES.contains(&fname.as_str()) {
                helpers.insert(crate::util::strip_unc_prefix(exe).to_string_lossy().to_string());
            }
        }
    }
}

fn scan_epic(found: &mut HashMap<&'static str, HashSet<String>>, helpers: &mut HashSet<String>) {
    let program_data = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".into());
    let manifest_dir = PathBuf::from(program_data)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");

    if let Ok(entries) = std::fs::read_dir(&manifest_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("item") {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else { continue };

            let install_location = json.get("InstallLocation").and_then(|v| v.as_str()).unwrap_or("");
            let launch_exe = json.get("LaunchExecutable").and_then(|v| v.as_str()).unwrap_or("");
            if install_location.is_empty() || launch_exe.is_empty() {
                continue;
            }

            let full_path = PathBuf::from(install_location).join(launch_exe);
            if !full_path.exists() {
                continue;
            }
            let full_path_str = full_path.to_string_lossy().to_string();
            let display = json.get("DisplayName").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let launch_exe_lower = launch_exe.to_lowercase();

            let matched_key = KNOWN_GAMES.iter().find(|d| {
                display.contains(&d.display_name.to_lowercase())
                    || d.match_names.iter().any(|n| *n == launch_exe_lower)
            }).map(|d| d.key);

            if let Some(key) = matched_key {
                found.entry(key).or_default().insert(full_path_str);
            } else {
                helpers.insert(full_path_str);
            }
        }
    }

    if let Ok(program_files) = std::env::var("ProgramFiles(x86)").or_else(|_| std::env::var("ProgramFiles")) {
        let launcher = PathBuf::from(program_files)
            .join("Epic Games")
            .join("Launcher")
            .join("Portal")
            .join("Binaries")
            .join("Win64")
            .join("EpicGamesLauncher.exe");
        if launcher.exists() {
            helpers.insert(launcher.to_string_lossy().to_string());
        }
    }
}

fn collect_exe_strings(value: &serde_json::Value, out: &mut HashSet<String>) {
    match value {
        serde_json::Value::String(s) => {
            if s.to_lowercase().ends_with(".exe") && PathBuf::from(s).exists() {
                out.insert(s.clone());
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                collect_exe_strings(v, out);
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                collect_exe_strings(v, out);
            }
        }
        _ => {}
    }
}

fn scan_riot(found: &mut HashMap<&'static str, HashSet<String>>, helpers: &mut HashSet<String>) {
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let config_path = PathBuf::from(&local_appdata)
            .join("Riot Games")
            .join("Riot Client")
            .join("Config")
            .join("RiotClientInstalls.json");

        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                collect_exe_strings(&json, helpers);
            }
        }
    }

    for base in ["C:\\Riot Games", "C:\\Program Files\\Riot Vanguard", "C:\\Program Files (x86)\\Riot Vanguard"] {
        let base_path = PathBuf::from(base);
        if !base_path.exists() {
            continue;
        }
        for exe in walk_for_exes(&base_path, 6, 10000) {
            let fname = exe.file_name().and_then(|f| f.to_str()).unwrap_or("").to_lowercase();
            let path_str = exe.to_string_lossy().to_string();

            if let Some(def) = KNOWN_GAMES.iter().find(|d| d.match_names.contains(&fname.as_str())) {
                found.entry(def.key).or_default().insert(path_str);
            } else if HELPER_PROCESS_NAMES.contains(&fname.as_str()) {
                helpers.insert(path_str);
            }
        }
    }
}

fn scan_battlenet(helpers: &mut HashSet<String>) {
    for base in ["C:\\Program Files (x86)\\Battle.net", "C:\\Program Files\\Battle.net"] {
        let exe = PathBuf::from(base).join("Battle.net.exe");
        if exe.exists() {
            helpers.insert(exe.to_string_lossy().to_string());
        }
        let agent = PathBuf::from(base).join("Agent.exe");
        if agent.exists() {
            helpers.insert(agent.to_string_lossy().to_string());
        }
    }
}

fn scan_shared_anticheat(helpers: &mut HashSet<String>) {
    let candidates = [
        "C:\\Program Files (x86)\\EasyAntiCheat\\EasyAntiCheat.exe",
        "C:\\Program Files\\EasyAntiCheat\\EasyAntiCheat.exe",
        "C:\\Program Files (x86)\\Common Files\\BattlEye\\BEService.exe",
        "C:\\Program Files\\Common Files\\BattlEye\\BEService.exe",
    ];
    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            helpers.insert(p.to_string_lossy().to_string());
        }
    }
}

fn scan_running_processes(found: &mut HashMap<&'static str, HashSet<String>>, helpers: &mut HashSet<String>) {
    let Ok(processes) = crate::util::enumerate_processes_with_paths() else { return };

    for (_, name, path) in processes {
        let lname = name.to_lowercase();
        let fname_exe = format!("{}.exe", lname);

        if let Some(def) = KNOWN_GAMES
            .iter()
            .find(|d| d.match_names.iter().any(|n| *n == fname_exe || *n == lname))
        {
            found.entry(def.key).or_default().insert(path.clone());
            continue;
        }

        if HELPER_PROCESS_NAMES.iter().any(|n| *n == fname_exe || *n == lname) {
            helpers.insert(path);
        }
    }
}

fn vdf_value(content: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with(&needle) {
            continue;
        }
        let rest = &trimmed[needle.len()..];
        let start = rest.find('\"')? + 1;
        let tail = &rest[start..];
        let end = tail.find('\"')?;
        return Some(tail[..end].replace("\\\\", "\\"));
    }
    None
}

fn steam_library_games() -> Vec<LibraryGame> {
    let mut games = Vec::new();
    for lib in get_steam_library_paths() {
        let apps = PathBuf::from(&lib).join("steamapps");
        let Ok(entries) = std::fs::read_dir(&apps) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_manifest = path
                .file_name()
                .and_then(|f| f.to_str())
                .map(|f| f.starts_with("appmanifest_") && f.ends_with(".acf"))
                .unwrap_or(false);
            if !is_manifest {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&path) else { continue };
            let Some(install_dir) = vdf_value(&content, "installdir") else { continue };
            let name = vdf_value(&content, "name").unwrap_or_else(|| install_dir.clone());
            let full = apps.join("common").join(&install_dir);
            if full.is_dir() {
                games.push(LibraryGame { name, dir: full, launcher: "Steam" });
            }
        }
    }
    games
}

fn epic_library_games() -> Vec<LibraryGame> {
    let mut games = Vec::new();
    let program_data = std::env::var("PROGRAMDATA").unwrap_or_else(|_| "C:\\ProgramData".into());
    let manifest_dir = PathBuf::from(program_data)
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");
    let Ok(entries) = std::fs::read_dir(&manifest_dir) else { return games };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("item") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else { continue };
        let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else { continue };
        let location = json.get("InstallLocation").and_then(|v| v.as_str()).unwrap_or("");
        if location.is_empty() {
            continue;
        }
        let dir = PathBuf::from(location);
        if !dir.is_dir() {
            continue;
        }
        let name = json
            .get("DisplayName")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| dir.file_name().and_then(|f| f.to_str()).unwrap_or("Unknown"))
            .to_string();
        games.push(LibraryGame { name, dir, launcher: "Epic Games" });
    }
    games
}

fn xbox_library_games() -> Vec<LibraryGame> {
    let mut games = Vec::new();
    for letter in 'C'..='Z' {
        let root = PathBuf::from(format!("{}:\\XboxGames", letter));
        let Ok(entries) = std::fs::read_dir(&root) else { continue };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let name = dir.file_name().and_then(|f| f.to_str()).unwrap_or("").to_string();
            if name.is_empty() {
                continue;
            }
            let content = dir.join("Content");
            let target = if content.is_dir() { content } else { dir };
            games.push(LibraryGame { name, dir: target, launcher: "Xbox" });
        }
    }
    games
}

pub fn detect_installed_games() -> Vec<DetectedGame> {
    let mut found: HashMap<&'static str, HashSet<String>> = HashMap::new();
    let mut generic_helpers: HashSet<String> = HashSet::new();

    scan_steam(&mut found, &mut generic_helpers);
    scan_epic(&mut found, &mut generic_helpers);
    scan_riot(&mut found, &mut generic_helpers);
    scan_battlenet(&mut generic_helpers);
    scan_shared_anticheat(&mut generic_helpers);
    scan_running_processes(&mut found, &mut generic_helpers);

    let mut results: Vec<DetectedGame> = Vec::new();

    for def in KNOWN_GAMES {
        if let Some(paths) = found.get(def.key) {
            if !paths.is_empty() {
                let mut exe_paths: Vec<String> = paths.iter().cloned().collect();
                exe_paths.sort();
                results.push(DetectedGame {
                    key: def.key.to_string(),
                    display_name: def.display_name.to_string(),
                    exe_paths,
                    recommended: true,
                    launcher: "Detected".to_string(),
                    install_dir: String::new(),
                });
            }
        }
    }

    if !generic_helpers.is_empty() {
        let mut exe_paths: Vec<String> = generic_helpers.into_iter().collect();
        exe_paths.sort();
        results.push(DetectedGame {
            key: "helpers".to_string(),
            display_name: "Launchers & Anti-Cheat services".to_string(),
            exe_paths,
            recommended: false,
            launcher: "System".to_string(),
            install_dir: String::new(),
        });
    }

    let mut claimed: HashSet<String> = HashSet::new();
    for game in &results {
        for exe in &game.exe_paths {
            claimed.insert(exe.to_lowercase());
        }
    }

    let mut library: Vec<LibraryGame> = Vec::new();
    library.extend(steam_library_games());
    library.extend(epic_library_games());
    library.extend(xbox_library_games());

    let mut seen_dirs: HashSet<String> = HashSet::new();
    for entry in library {
        let dir_key = entry.dir.to_string_lossy().to_lowercase();
        if !seen_dirs.insert(dir_key.clone()) {
            continue;
        }
        let exes: Vec<String> = pick_exes(&entry.dir)
            .into_iter()
            .filter(|e| !claimed.contains(&e.to_lowercase()))
            .collect();
        if exes.is_empty() {
            continue;
        }
        for exe in &exes {
            claimed.insert(exe.to_lowercase());
        }
        results.push(DetectedGame {
            key: format!("lib:{}", dir_key),
            display_name: entry.name,
            exe_paths: exes,
            recommended: false,
            launcher: entry.launcher.to_string(),
            install_dir: entry.dir.to_string_lossy().to_string(),
        });
    }

    results
}

#[tauri::command]
pub async fn scan_installed_games() -> Result<Vec<DetectedGame>, String> {
    tokio::task::spawn_blocking(detect_installed_games)
        .await
        .map_err(|e| e.to_string())
}
