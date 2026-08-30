mod actions;
mod store;
mod brand;
mod shell;
mod tui;

use clap::{Args, Parser, Subcommand};

use actions::ConnectRequest;

#[derive(Parser)]
#[command(
    name = "wawity",
    version,
    about = "Wawity VPN command-line client and interactive console",
    long_about = "Run without arguments to launch the interactive console."
)]
struct Cli {
    #[arg(long, global = true, help = "Machine-readable JSON output")]
    json: bool,

    #[arg(long, hide = true)]
    autostart: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Живая панель мониторинга (TUI)")]
    Dash,
    #[command(about = "Manage subscriptions")]
    Sub {
        #[command(subcommand)]
        action: SubCommand,
    },
    #[command(about = "Connect to a server (name or 'fastest')")]
    Connect(ConnectArgs),
    #[command(about = "Disconnect the active tunnel")]
    Disconnect,
    #[command(about = "Switch to another server without full teardown")]
    Switch(SwitchArgs),
    #[command(about = "List available servers")]
    Servers(ServersArgs),
    #[command(about = "Show connection status")]
    Status(StatusArgs),
    #[command(about = "Repair network state after a crash")]
    Repair,
    #[command(about = "Launch the interactive console")]
    Console,
}

#[derive(Subcommand)]
enum SubCommand {
    #[command(about = "Add a subscription URL")]
    Add {
        url: String,
        #[arg(long)]
        name: Option<String>,
    },
    #[command(about = "List subscriptions")]
    List,
    #[command(about = "Refresh subscriptions")]
    Refresh {
        #[arg(long)]
        id: Option<String>,
    },
    #[command(about = "Remove a subscription by id")]
    Rm { id: String },
}

#[derive(Args)]
struct ConnectArgs {
    #[arg(default_value = "fastest", help = "Server name or 'fastest'")]
    server: String,
    #[arg(long, help = "Entry server for multi-hop")]
    entry: Option<String>,
    #[arg(long, help = "Disable the kill switch for this session")]
    no_killswitch: bool,
    #[arg(long, help = "Disable post-quantum key exchange")]
    no_quantum: bool,
    #[arg(long, help = "Run in the foreground (Ctrl+C disconnects)")]
    fg: bool,
}

#[derive(Args)]
struct SwitchArgs {
    server: String,
    #[arg(long)]
    entry: Option<String>,
}

#[derive(Args)]
struct ServersArgs {
    #[arg(long, help = "Measure latency to each server")]
    ping: bool,
}

#[derive(Args)]
struct StatusArgs {
    #[arg(long, help = "Poll status every 2 seconds")]
    watch: bool,
}

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    let result = match cli.command {
        None => {
            if cli.autostart {
                autostart_reconnect(json)
            } else {
                shell::run()
            }
        }
        Some(Command::Console) => shell::run(),
        Some(Command::Dash) => tui::run(),
        Some(Command::Sub { action }) => match action {
            SubCommand::Add { url, name } => actions::cmd_sub_add(url, name, json),
            SubCommand::List => actions::cmd_sub_list(json),
            SubCommand::Refresh { id } => actions::cmd_sub_refresh(id, json),
            SubCommand::Rm { id } => actions::cmd_sub_rm(id, json),
        },
        Some(Command::Connect(args)) => {
            let settings = store::load_settings();
            actions::cmd_connect(
                ConnectRequest {
                    target: args.server,
                    entry: args.entry,
                    kill_switch: !args.no_killswitch && settings.kill_switch,
                    quantum_resistant: !args.no_quantum && settings.quantum_resistant,
                    foreground: args.fg,
                },
                json,
            )
        }
        Some(Command::Disconnect) => actions::cmd_disconnect(json),
        Some(Command::Switch(args)) => cmd_switch(args, json),
        Some(Command::Servers(args)) => actions::cmd_servers(args.ping, json),
        Some(Command::Status(args)) => actions::cmd_status(json, args.watch),
        Some(Command::Repair) => actions::cmd_repair(json),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_switch(args: SwitchArgs, json: bool) -> Result<(), String> {
    let settings = store::load_settings();
    let entries = actions::collect_servers()?;
    let idx = if args.server.eq_ignore_ascii_case("fastest") {
        actions::pick_fastest(&entries).ok_or("No reachable server")?
    } else {
        actions::find_server(&entries, &args.server)
            .ok_or_else(|| format!("Server not found: {}", args.server))?
    };
    let exit = &entries[idx];
    let entry_name = args.entry.clone().or_else(|| settings.entry_server.clone());
    let (entry_url, entry_display) = match &entry_name {
        Some(name) if !name.trim().is_empty() => {
            let e_idx = actions::find_server(&entries, name)
                .ok_or_else(|| format!("Entry server not found: {}", name))?;
            (
                Some(entries[e_idx].server.url.clone()),
                Some(entries[e_idx].server.name.clone()),
            )
        }
        _ => (None, None),
    };
    let session = wawity_core::ops::Session::new()?;
    session.set_privacy(actions::privacy_from(&settings));
    session.switch(
        &exit.server.url,
        entry_url,
        Some(exit.server.name.clone()),
        entry_display,
        settings.bypass_apps.clone(),
        settings.quantum_resistant,
    )?;
    let pid = session.status().pid;
    let detached = store::DetachedState {
        pid,
        kill_switch: settings.kill_switch,
        exit_server: Some(exit.server.name.clone()),
        entry_server: entry_name,
        always_on: settings.kill_switch,
        started_at: chrono::Utc::now().timestamp(),
    };
    store::save_detached(&detached)?;
    std::thread::sleep(std::time::Duration::from_secs(3));
    std::mem::forget(session);
    if json {
        println!("{{\"switched\":true}}");
    } else {
        println!("Switched to {}.", exit.server.name);
    }
    Ok(())
}

fn autostart_reconnect(json: bool) -> Result<(), String> {
    let detached = store::load_detached();
    let server = detached.and_then(|d| d.exit_server).unwrap_or_else(|| "fastest".into());
    let settings = store::load_settings();
    actions::cmd_connect(
        ConnectRequest {
            target: server,
            entry: settings.entry_server.clone(),
            kill_switch: settings.kill_switch,
            quantum_resistant: settings.quantum_resistant,
            foreground: false,
        },
        json,
    )
}
