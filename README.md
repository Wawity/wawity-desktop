# Wawity VPN

**Wawity** is a Windows VPN client built on top of [sing-box](https://github.com/SagerNet/sing-box). It ships as a desktop GUI app (Tauri + Vue 3) and a companion CLI/TUI client, both powered by a shared Rust engine that handles protocol parsing, process supervision, and Windows Firewall–based traffic lockdown.

🇷🇺 Русская версия: [README.ru.md](./README.ru.md)

> A separate mobile port lives in a sibling project, `wawity-android` (Tauri 2 + Kotlin), and is not part of this repository.

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Project Structure](#project-structure)
- [Requirements](#requirements)
- [Getting Started](#getting-started)
  - [Frontend / desktop app (dev mode)](#frontend--desktop-app-dev-mode)
  - [CLI client](#cli-client)
  - [One-click build script](#one-click-build-script)
  - [Building the installer manually](#building-the-installer-manually)
- [Telemetry](#telemetry)
- [Localization](#localization)
- [License](#license)
- [Disclaimer](#disclaimer)

---

## Overview

Wawity connects to proxy servers described by standard subscription links (VLESS, VMess, Trojan, Shadowsocks, Hysteria2) and routes traffic through [sing-box](https://github.com/SagerNet/sing-box) via a TUN interface (`wintun.dll`). On top of that it adds a full desktop experience: kill switch, always-on lockdown, multi-hop routing, split tunneling, ad/tracker blocking, system tray control, hotkeys, Discord Rich Presence, and a bilingual (EN/RU) UI with an animated space-themed background.

The product ships in two flavors from the same codebase:

- **Wawity Desktop** — the full GUI app (`wawity-app.exe`), built with Tauri.
- **Wawity CLI** — a lightweight console/TUI client (`wawity.exe`) for headless or server use, built with `ratatui`.

## Features

**Core VPN engine**
- Protocol support: VLESS, VMess, Trojan, Shadowsocks, Hysteria2 (subscription URLs are auto-detected and parsed).
- Subscription import by pasting a URL; automatic fallback across multiple User-Agents and content types when fetching.
- Server ping/latency measurement and geolocation for the server list.
- Post-quantum encryption option: hybrid **X25519 + ML-KEM768** key exchange.

**Network protection**
- **Kill Switch** — blocks all traffic if the VPN tunnel drops.
- **Always-On lockdown** — blocks all internet access until a server is connected, enforced through Windows Firewall (COM `INetFwPolicy2` API) with a background watchdog that detects and repairs firewall rule drift.
- **DNS leak guard** — blocks LAN DNS, LLMNR, mDNS, and NBNS traffic that could bypass the tunnel.
- **Multi-hop routing** — chain traffic through an entry server and an exit server (`entry → exit → destination`).
- **Split tunneling** — include/exclude specific apps, domains, or IP ranges from the tunnel; the app scans installed games and applications to make picking them easier.
- **Ad & tracker blocking** — ships with `sing-box` geosite rulesets (`geosite-category-ads-all`, `geosite-private`).

**Desktop experience**
- System tray with quick connect/disconnect, reconnect, and network repair actions, plus a tray popup mini-UI.
- Configurable global hotkeys (works even when the window is hidden).
- Native desktop notifications.
- Discord Rich Presence integration (optionally shows connected server/subscription).
- Built-in network tools: server reachability checks, speed test, and DNS/IP leak diagnostics (`src/views/extra/`).
- Auto-start on boot, with an option to launch minimized to tray.
- Animated “space objects” background (black hole / pulsar / neutron star / nebula) with a togglable detail level (simple vs. cinematic) for lower-end GPUs.
- Bilingual interface: English and Russian, switchable at runtime.

**CLI client**
- Interactive terminal UI (`ratatui` + `crossterm`) with the same core engine as the desktop app, plus a scriptable `clap`-based command surface.

## Architecture

The repository is a Cargo workspace with three members plus a Vue frontend and a standalone installer project:

```
Cargo.toml                 # workspace root (resolver = "2")
crates/
├── wawity-core/           # shared engine: config parsing/generation, sing-box
│                          # process supervision, Windows Firewall lockdown,
│                          # routing/TUN/QoS/network helpers
└── wawity-cli/            # console binary "wawity": clap CLI + ratatui TUI
src-tauri/                 # Tauri 1.8 desktop app (binary "wawity-app")
│                          # Win32 integration: firewall COM API, global
│                          # hotkeys, DWM window effects, system tray,
│                          # Discord Rich Presence, installed-app/game scan
src/                       # Vue 3 + Pinia + Vue Router + TypeScript frontend
installer/                 # standalone Rust project that builds the
                            # Windows installer (WawitySetup-*.exe)
telemetry-relay/           # optional Node.js relay for opt-in telemetry
```

- **`wawity-core`** is protocol- and platform-agnostic where possible; it owns subscription parsing (`config::parser`), sing-box config generation (`config::generator`), the process supervisor, and Windows network/firewall management.
- **`src-tauri`** wires the engine into a native window: it registers all Tauri commands consumed by the frontend, manages splash/main/tray/notification windows, and owns the Win32-specific pieces (firewall rules, hotkeys, window chrome).
- **`src`** is a standard Vite + Vue 3 SPA. State lives in a single Pinia store (`stores/vpn.ts`) that talks to the backend exclusively through `@tauri-apps/api` `invoke`/`listen`.
- **`installer/`** is a separate Cargo project (excluded from the main workspace) that stages the WebView2 bootstrapper, the built binary, `sing-box`, `wintun.dll`, and the geosite rulesets into a single NSIS-style installer executable.
- **`telemetry-relay/`** is a small, independent Node.js HTTP service; the desktop app only talks to it if telemetry is enabled in Settings.

## Project Structure

```
wawity/
├── crates/
│   ├── wawity-core/        Rust engine (library)
│   └── wawity-cli/         Rust CLI/TUI binary
├── src-tauri/
│   ├── src/                Tauri commands & Windows integration
│   ├── binaries/           bundled sing-box-x86_64.exe, wintun.dll
│   ├── rulesets/           geosite .srs rule files
│   └── icons/               app & tray icons
├── src/
│   ├── components/          Vue components (UI + animated backgrounds)
│   ├── views/                Connection / Servers / Analysis / Settings
│   ├── views/extra/          Reachability / SpeedTest / Leaks utilities
│   ├── stores/                Pinia store (vpn.ts)
│   ├── i18n/                   en.ts / ru.ts locales
│   ├── composables/
│   ├── lib/                     geo.ts, telemetry.ts helpers
│   └── router/
├── installer/                installer Cargo project + NSIS-like UI
├── telemetry-relay/          optional telemetry relay (Node.js)
├── .github/workflows/        release CI (builds and attaches installers)
├── build.bat                  interactive build menu
├── fix-cargo-mirror.bat        disables a global cargo registry mirror
├── package.json
├── Cargo.toml                  workspace manifest
└── tsconfig*.json / vite.config.ts
```

## Requirements

- Windows 10/11, x64.
- The app requests administrator privileges on launch (it self-elevates via UAC) because it manages the Windows Firewall and network stack directly.
- To build from source: Node.js (npm), the Rust toolchain (`cargo`), and — on Windows — the MSVC Build Tools for linking.

## Getting Started

### Frontend / desktop app (dev mode)

```bash
npm install
npm run dev          # Vite dev server
# in a second terminal
cargo build -p wawity          # debug build of the Tauri app
```

### CLI client

```bash
cargo build -p wawity-cli --release
target/release/wawity.exe
```

### One-click build script

`build.bat` provides an interactive menu for producing signed-ready installers:

```
[1] DESKTOP   — frontend + Rust + installer  → dist-build\WawitySetup-Desktop.exe
[2] CLI       — binary + installer           → dist-build\WawitySetup-CLI.exe
[3] Both products in sequence
[4] Clean build artifacts
[5] Diagnose cargo registry/mirror/network issues
```

The script auto-detects a global cargo registry mirror (`replace-with`) and, if found, redirects the build to an isolated `CARGO_HOME` (`.cargo-home/`) so the mirror doesn't interfere. Use `fix-cargo-mirror.bat` if you'd rather disable the mirror in your global `~/.cargo/config.toml` entirely.

### Building the installer manually

The installer packaging step (`installer\`) expects these files to exist before it runs:

- `installer/payload/MicrosoftEdgeWebView2Setup.exe` — WebView2 Evergreen Bootstrapper (bundled in the repo).
- `src-tauri/binaries/sing-box-x86_64.exe` and `src-tauri/binaries/wintun.dll` — already bundled in this repo.
- `src-tauri/rulesets/*.srs` — geosite rules for ad/tracker blocking, already bundled.

`build.bat` stages these into `installer/payload/app.zip` and then builds the `installer` Cargo project, which produces `WawitySetup-Desktop.exe` / `WawitySetup-CLI.exe` in `dist-build\`.

## Telemetry

Wawity can optionally report anonymous usage events (connect/disconnect, errors) to help diagnose issues. This is a Settings toggle, off or on depending on your build defaults — check the **Settings** page in the app to confirm the current state. When enabled, events are sent to a self-hosted relay (`telemetry-relay/server.mjs`), which authenticates requests with an ingest key, rate-limits by IP, and periodically forwards a digest to a Telegram chat via a bot token. No telemetry code path runs unless the relay is deployed and configured by whoever builds/operates that instance.

Relay environment variables:

| Variable | Purpose |
|---|---|
| `TG_BOT_TOKEN` | Telegram bot token used to deliver digests |
| `TG_CHAT_ID` | Destination chat for digests |
| `INGEST_KEY` | Shared secret the desktop app must present |
| `PORT` | HTTP port (default `8787`) |
| `DIGEST_MINUTES` | Digest interval, minimum 5 minutes |

## Localization

The UI ships with English (`src/i18n/en.ts`) and Russian (`src/i18n/ru.ts`) translations, switchable at runtime from Settings. Adding a new language means adding a new locale file with the same key structure and registering it in `src/i18n/index.ts`.

## License

Wawity is released under the [MIT License](./LICENSE). You are free to use, copy, modify, merge, publish, distribute, sublicense, and sell copies of the software, provided the copyright notice and permission notice are included in all copies or substantial portions of it.

## Disclaimer

Wawity is a general-purpose VPN/proxy client. You are responsible for complying with the laws and terms of service applicable in your jurisdiction and with any service you connect to through it. The maintainers are not responsible for misuse.
