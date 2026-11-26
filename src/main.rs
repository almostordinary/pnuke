use clap::{CommandFactory, Parser};
use colored::*;
use std::process::ExitCode;

mod engine;
mod resolver;
mod tui;
mod types;

use engine::{find_processes_on_port, get_all_listeners, nuke_port};
use resolver::{ALIASES, get_protection, resolve_all_targets};
use types::InspectResult;

#[derive(Parser, Debug)]
#[command(
    name = "pnuke",
    author = "Almost Ordinary <hello@almostordinary.dev>",
    version,
    about = "⚡ The sub-millisecond port killer and listener cockpit for developers.",
    long_about = "⚡ pnuke is an ultra-fast, cross-platform CLI tool built in Rust to instantly find and terminate processes locking your development ports.\n\nRun 'pnuke 3000' or 'pnuke vite' to nuke ports in <2ms, or run 'pnuke' with no arguments to launch the interactive terminal cockpit."
)]
struct Cli {
    /// Port(s), aliases (e.g. 'vite', 'next', 'postgres'), or ranges (e.g. '3000..3005')
    #[arg(value_name = "TARGETS")]
    targets: Vec<String>,

    /// Force immediate termination (SIGKILL) without graceful grace period
    #[arg(short, long)]
    force: bool,

    /// Check and inspect what is running without terminating
    #[arg(short, long)]
    check: bool,

    /// Non-interactively list all active listening ports
    #[arg(short, long)]
    list: bool,

    /// Explicitly launch the interactive terminal UI
    #[arg(short, long)]
    interactive: bool,

    /// List all supported framework and service port aliases
    #[arg(long)]
    aliases: bool,

    /// Allow terminating protected system ports (e.g. port 22 SSH, port 53 DNS)
    #[arg(long)]
    danger: bool,

    /// Generate shell auto-completions (bash, zsh, fish, powershell)
    #[arg(long, value_enum, value_name = "SHELL")]
    generate_completion: Option<clap_complete::Shell>,

    /// Output results as JSON (ideal for piping or AI agents)
    #[arg(long)]
    json: bool,

    /// Suppress all informational output except errors
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // 1. Shell Completion Generator
    if let Some(shell) = cli.generate_completion {
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "pnuke", &mut std::io::stdout());
        return ExitCode::SUCCESS;
    }

    // 2. List Supported Aliases
    if cli.aliases {
        println!(
            "\n  {} {}",
            "⚡ PNUKE".black().on_cyan().bold(),
            "Supported Framework & Service Aliases".bold()
        );
        println!(
            "  {:<18} {:<24}",
            "ALIAS".white().bold(),
            "DEFAULT PORT(S)".white().bold()
        );
        println!("  {}", "─".repeat(45).dimmed());

        for (alias, ports) in ALIASES {
            let port_str = ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            println!(
                "  {:<18} {:<24}",
                alias.green().bold(),
                port_str.cyan().bold()
            );
        }
        println!(
            "\n  {} Try running '{}'\n",
            "Tip:".yellow().bold(),
            "pnuke vite".cyan()
        );
        return ExitCode::SUCCESS;
    }

    // 3. Interactive TUI Mode (Default when no targets or flags)
    if cli.interactive || (cli.targets.is_empty() && !cli.list) {
        if cli.json {
            eprintln!(
                "{}",
                "Error: --json cannot be used with interactive mode".red()
            );
            return ExitCode::FAILURE;
        }

        if let Err(e) = tui::run_tui() {
            eprintln!(
                "{} Failed to launch interactive TUI: {}",
                "✖".red().bold(),
                e
            );
            return ExitCode::FAILURE;
        }
        return ExitCode::SUCCESS;
    }

    // 4. Non-interactive List Mode
    if cli.list {
        match get_all_listeners() {
            Ok(listeners) => {
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&listeners).unwrap());
                    return ExitCode::SUCCESS;
                }

                if listeners.is_empty() {
                    println!("{}", "No active listening ports detected.".yellow());
                    return ExitCode::SUCCESS;
                }

                println!(
                    "\n  {} {}",
                    "⚡ PNUKE".black().on_cyan().bold(),
                    format!("Active Listening Ports ({})", listeners.len()).bold()
                );
                println!(
                    "  {:<8} {:<8} {:<10} {:<24} {:<20}",
                    "PORT".white().bold(),
                    "PROTO".white().bold(),
                    "PID".white().bold(),
                    "PROCESS".white().bold(),
                    "ADDRESS".white().bold()
                );
                println!("  {}", "─".repeat(72).dimmed());

                for p in listeners {
                    println!(
                        "  {:<8} {:<8} {:<10} {:<24} {:<20}",
                        p.port.to_string().cyan().bold(),
                        p.protocol.yellow(),
                        p.pid.to_string().magenta(),
                        p.name.green().bold(),
                        format!("{}:{}", p.ip, p.port).dimmed()
                    );
                }
                println!();
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("{} {}", "✖".red().bold(), e);
                return ExitCode::FAILURE;
            }
        }
    }

    // 5. Target Resolution (Aliases, Ranges, Ports)
    let ports = match resolve_all_targets(&cli.targets) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", "✖".red().bold(), e);
            return ExitCode::FAILURE;
        }
    };

    // 6. Safety Shield: Check for Protected Ports
    if !cli.danger && !cli.force && !cli.check {
        for &port in &ports {
            if let Some(service) = get_protection(port) {
                eprintln!(
                    "{} Safety Shield: Port {} is a protected system port ({}).\n  Pass '{}' to force termination.",
                    "🛡️".yellow(),
                    port.to_string().cyan().bold(),
                    service.bold(),
                    "--danger".red().bold()
                );
                return ExitCode::FAILURE;
            }
        }
    }

    // 7. Inspect / Check Mode
    if cli.check {
        let mut inspect_results = Vec::new();

        for &port in &ports {
            match find_processes_on_port(port) {
                Ok(procs) => {
                    let occupied = !procs.is_empty();

                    if cli.json {
                        inspect_results.push(InspectResult {
                            port,
                            occupied,
                            processes: procs,
                        });
                    } else if !cli.quiet {
                        if procs.is_empty() {
                            println!(
                                "  {} Port {} is free (no active listener).",
                                "✔".green().bold(),
                                port.to_string().cyan().bold()
                            );
                        } else {
                            for p in procs {
                                println!(
                                    "  {} Port {} is occupied by '{}' (PID: {}, Protocol: {})",
                                    "●".yellow().bold(),
                                    port.to_string().cyan().bold(),
                                    p.name.green().bold(),
                                    p.pid.to_string().magenta().bold(),
                                    p.protocol.yellow()
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  {} Port {}: {}", "✖".red().bold(), port, e);
                }
            }
        }

        if cli.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&inspect_results).unwrap()
            );
        }

        return ExitCode::SUCCESS;
    }

    // 8. Nuke Mode (Default execution)
    let mut all_results = Vec::new();
    let mut any_failed = false;

    for &port in &ports {
        let results = nuke_port(port, cli.force);
        for res in &results {
            if !res.success {
                any_failed = true;
            }
        }
        all_results.extend(results);
    }

    if cli.json {
        println!("{}", serde_json::to_string_pretty(&all_results).unwrap());
        return if any_failed {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }

    if !cli.quiet {
        for res in all_results {
            if res.success {
                println!(
                    "  {} Nuked '{}' (PID: {}) on port {} in {:.2}ms",
                    "✔".green().bold(),
                    res.process_name.green().bold(),
                    res.pid.to_string().magenta(),
                    res.port.to_string().cyan().bold(),
                    res.duration_ms
                );
            } else {
                println!(
                    "  {} Failed on port {}: {}",
                    "✖".red().bold(),
                    res.port.to_string().cyan().bold(),
                    res.message.yellow()
                );
            }
        }
    }

    if any_failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
