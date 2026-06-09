//! Terminal/PTMX collector
//! Tracks pseudo-terminal (ptmx) usage across the system

use std::collections::HashMap;
use std::time::Duration;

use crate::types::{TerminalInfo, TerminalProcess};

/// Collect terminal/PTMX information using lsof
pub async fn collect_terminal_info() -> TerminalInfo {
    let output = tokio::time::timeout(
        Duration::from_millis(800),
        tokio::process::Command::new("lsof")
            .args(["-nP", "/dev/ttys*", "/dev/ptmx"])
            .output(),
    )
    .await;

    match output {
        Ok(Ok(output)) if output.status.success() => {
            parse_lsof_output(&String::from_utf8_lossy(&output.stdout))
        }
        _ => {
            let count = count_ptmx_basic();
            TerminalInfo {
                total_ptmx_count: count,
                local_ptmx_count: count,
                terminal_processes: Vec::new(),
            }
        }
    }
}

fn parse_lsof_output(output: &str) -> TerminalInfo {
    let mut process_ptys: HashMap<u32, (String, u32)> = HashMap::new();
    let mut total_count = 0u32;

    for line in output.lines().skip(1) {
        // lsof output format: COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 9 {
            if let Ok(pid) = parts[1].parse::<u32>() {
                let name = parts[0].to_string();
                total_count += 1;

                process_ptys
                    .entry(pid)
                    .and_modify(|(_, count)| *count += 1)
                    .or_insert((name, 1));
            }
        }
    }

    // Convert to sorted list by pty count (descending)
    let mut processes: Vec<TerminalProcess> = process_ptys
        .into_iter()
        .map(|(pid, (name, count))| TerminalProcess {
            pid,
            name,
            pty_count: count,
        })
        .collect();

    processes.sort_by_key(|p| std::cmp::Reverse(p.pty_count));
    processes.truncate(10); // Keep top 10

    // Count local terminals (common terminal emulators)
    let local_terminals = [
        "iTerm",
        "Terminal",
        "alacritty",
        "kitty",
        "wezterm",
        "ghostty",
        "warp",
    ];
    let local_count = processes
        .iter()
        .filter(|p| {
            local_terminals
                .iter()
                .any(|term| p.name.to_lowercase().contains(&term.to_lowercase()))
        })
        .map(|p| p.pty_count)
        .sum();

    TerminalInfo {
        total_ptmx_count: total_count,
        local_ptmx_count: local_count,
        terminal_processes: processes,
    }
}

fn count_ptmx_basic() -> u32 {
    std::fs::read_dir("/dev")
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("ttys"))
                .count() as u32
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_info_default() {
        let info = TerminalInfo::default();
        assert_eq!(info.total_ptmx_count, 0);
        assert_eq!(info.local_ptmx_count, 0);
        assert!(info.terminal_processes.is_empty());
    }

    #[tokio::test]
    async fn test_collect_terminal_info() {
        // This test will only pass on macOS/Linux systems
        let info = collect_terminal_info().await;
        // Should not panic and return valid data
        println!("Terminal info: {:?}", info);
    }
}
