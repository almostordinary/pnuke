use crate::types::{KillResult, ProcessInfo};
use std::collections::HashSet;
use std::time::Instant;

/// Retrieves all active listening processes across TCP and UDP sockets.
pub fn get_all_listeners() -> Result<Vec<ProcessInfo>, String> {
    let raw_listeners =
        listeners::get_all().map_err(|e| format!("Failed to inspect sockets: {e}"))?;

    let mut seen = HashSet::new();
    let mut results = Vec::new();

    for l in raw_listeners {
        let port = l.socket.port();
        if port == 0 {
            continue;
        }

        let proto_str = match l.protocol {
            listeners::Protocol::TCP => "TCP",
            listeners::Protocol::UDP => "UDP",
        };

        let info = ProcessInfo {
            pid: l.process.pid,
            name: if l.process.name.is_empty() {
                "unknown".to_string()
            } else {
                l.process.name
            },
            path: l.process.path,
            port,
            protocol: proto_str.to_string(),
            ip: l.socket.ip().to_string(),
        };

        let key = (info.port, info.pid, info.protocol.clone());
        if seen.insert(key) {
            results.push(info);
        }
    }

    results.sort_by_key(|p| p.port);
    Ok(results)
}

/// Finds all listening processes for a specific port.
/// Uses fast-path targeted lookup first, falling back to full scan if needed.
pub fn find_processes_on_port(target_port: u16) -> Result<Vec<ProcessInfo>, String> {
    let mut matches = Vec::new();
    let mut found_fast = false;

    // Fast path: query targeted TCP socket
    if let Ok(p) = listeners::get_process_by_port(target_port, listeners::Protocol::TCP) {
        matches.push(ProcessInfo {
            pid: p.pid,
            name: if p.name.is_empty() {
                "unknown".to_string()
            } else {
                p.name
            },
            path: p.path,
            port: target_port,
            protocol: "TCP".to_string(),
            ip: "0.0.0.0".to_string(),
        });
        found_fast = true;
    }

    // Fast path: query targeted UDP socket
    if let Ok(p) = listeners::get_process_by_port(target_port, listeners::Protocol::UDP) {
        matches.push(ProcessInfo {
            pid: p.pid,
            name: if p.name.is_empty() {
                "unknown".to_string()
            } else {
                p.name
            },
            path: p.path,
            port: target_port,
            protocol: "UDP".to_string(),
            ip: "0.0.0.0".to_string(),
        });
        found_fast = true;
    }

    if found_fast {
        return Ok(matches);
    }

    // Fallback path: inspect full system listener table
    let all = get_all_listeners()?;
    let fallback_matches: Vec<ProcessInfo> =
        all.into_iter().filter(|p| p.port == target_port).collect();
    Ok(fallback_matches)
}

/// Terminates a process by its PID.
/// On Unix, it sends SIGTERM, checks immediately, and uses microsecond-level adaptive polling (50µs)
/// before falling back to SIGKILL. If `force` is true, sends SIGKILL immediately (~0.02ms).
#[cfg(unix)]
pub fn terminate_pid(pid: u32, force: bool) -> Result<(), String> {
    unsafe {
        // Check if process is alive
        if libc::kill(pid as libc::pid_t, 0) != 0 {
            return Err(format!(
                "Process {} does not exist or has already exited",
                pid
            ));
        }

        if force {
            if libc::kill(pid as libc::pid_t, libc::SIGKILL) == 0 {
                return Ok(());
            } else {
                return Err(format!(
                    "Failed to SIGKILL process {}: {}",
                    pid,
                    std::io::Error::last_os_error()
                ));
            }
        }

        // Graceful termination: send SIGTERM
        if libc::kill(pid as libc::pid_t, libc::SIGTERM) != 0 {
            return Err(format!(
                "Failed to send SIGTERM to process {}: {}",
                pid,
                std::io::Error::last_os_error()
            ));
        }

        // Immediate check: did the process terminate right away?
        if libc::kill(pid as libc::pid_t, 0) != 0 {
            return Ok(());
        }

        // Adaptive microsecond polling: start at 50µs (0.05ms), double up to 500µs, total max 15ms
        let mut sleep_micros = 50u64;
        let mut total_slept = 0u64;

        while total_slept < 15_000 {
            std::thread::sleep(std::time::Duration::from_micros(sleep_micros));
            total_slept += sleep_micros;

            if libc::kill(pid as libc::pid_t, 0) != 0 {
                return Ok(()); // Terminated cleanly
            }

            if sleep_micros < 500 {
                sleep_micros *= 2;
            }
        }

        // If still alive after 15ms, escalate to instant SIGKILL
        if libc::kill(pid as libc::pid_t, libc::SIGKILL) == 0 {
            Ok(())
        } else {
            Err(format!(
                "Process {} did not respond to SIGTERM, and SIGKILL failed: {}",
                pid,
                std::io::Error::last_os_error()
            ))
        }
    }
}

/// Cross-platform termination for non-Unix (e.g. Windows).
#[cfg(not(unix))]
pub fn terminate_pid(pid: u32, _force: bool) -> Result<(), String> {
    let mut sys =
        sysinfo::System::new().map_err(|e| format!("Failed to initialize system info: {e}"))?;
    let sys_pid = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[sys_pid]), true);

    if let Some(process) = sys.process(sys_pid) {
        if process.kill() {
            Ok(())
        } else {
            Err(format!("Failed to terminate process {}", pid))
        }
    } else {
        Err(format!("Process {} not found", pid))
    }
}

/// Nukes all processes listening on a given port and returns execution metrics.
pub fn nuke_port(port: u16, force: bool) -> Vec<KillResult> {
    let start = Instant::now();
    let procs = match find_processes_on_port(port) {
        Ok(p) => p,
        Err(e) => {
            let duration_ms = (start.elapsed().as_nanos() as f64) / 1_000_000.0;
            return vec![KillResult {
                port,
                pid: 0,
                process_name: "none".to_string(),
                success: false,
                message: e,
                duration_ms,
            }];
        }
    };

    if procs.is_empty() {
        let duration_ms = (start.elapsed().as_nanos() as f64) / 1_000_000.0;
        return vec![KillResult {
            port,
            pid: 0,
            process_name: "none".to_string(),
            success: false,
            message: format!("No process found listening on port {}", port),
            duration_ms,
        }];
    }

    let mut results = Vec::new();
    for p in procs {
        let kill_start = Instant::now();
        let res = terminate_pid(p.pid, force);
        let duration_ms = (kill_start.elapsed().as_nanos() as f64) / 1_000_000.0;

        match res {
            Ok(()) => results.push(KillResult {
                port,
                pid: p.pid,
                process_name: p.name,
                success: true,
                message: "Terminated successfully".to_string(),
                duration_ms,
            }),
            Err(err_msg) => results.push(KillResult {
                port,
                pid: p.pid,
                process_name: p.name,
                success: false,
                message: err_msg,
                duration_ms,
            }),
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_listeners_does_not_panic() {
        let listeners = get_all_listeners();
        assert!(listeners.is_ok(), "get_all_listeners should succeed");
    }

    #[test]
    fn test_nuke_nonexistent_port() {
        let results = nuke_port(65534, false);
        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert_eq!(results[0].port, 65534);
    }
}
