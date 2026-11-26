use clap::Parser;
use colored::*;
use std::process::ExitCode;

mod engine;
mod tui;
mod types;

use engine::{find_processes_on_port, get_all_listeners, nuke_port};
use types::InspectResult;

#[derive(Parser, Debug)]
#[command(
    name = "pnuke",
    author = "Almost Ordinary <hello@almostordinary.dev>",
    version,
    about = "⚡ The sub-millisecond port killer and listener cockpit for developers.",
    long_about = "⚡ pnuke is an ultra-fast, cross-platform CLI tool built in Rust to instantly find and terminate processes locking your development ports.\n\nRun 'pnuke 3000' to nuke port 3000 in <2ms, or run 'pnuke' with no arguments to launch the interactive terminal cockpit."
)]
struct Cli {
    /// Port(s) to inspect or nuke (e.g. 3000, 8080, 5432)
    #[arg(value_name = "PORTS")]
    ports: Vec<u16>,

    /// Force immediate termination (SIGKILL) without graceful grace period
    #[arg(short, long)]
    force: bool,

    /// Check and inspect what is running on the port without terminating
    #[arg(short, long)]
    check: bool,

    /// Non-interactively list all active listening ports
    #[arg(short, long)]
    list: bool,

    /// Explicitly launch the interactive terminal UI
    #[arg(short, long)]
    interactive: bool,

    /// Output results as JSON (ideal for piping or AI agents)
    #[arg(long)]
    json: bool,

    /// Suppress all informational output except errors
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // 1. Explicit or default Interactive TUI Mode
    if cli.interactive || (cli.ports.is_empty() && !cli.list) {
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

    // 2. Non-interactive List Mode
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

    // 3. Inspect / Check Mode
    if cli.check {
        let mut inspect_results = Vec::new();

        for &port in &cli.ports {
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

    // 4. Nuke Mode (Default when ports specified)
    let mut all_results = Vec::new();
    let mut any_failed = false;

    for &port in &cli.ports {
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
