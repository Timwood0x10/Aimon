//! Bounded directory usage collector.
//! Scans a small set of user-facing directories with strict entry and depth
//! limits so Storage view can show top consumers without blocking the TUI.

use crate::types::DirectoryUsageInfo;
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const MAX_SCAN_DEPTH: usize = 4;
const MAX_SCAN_ENTRIES: u64 = 25_000;
const MAX_TOP_DIRECTORIES: usize = 8;

#[derive(Debug, Clone, Copy)]
struct ScanLimits {
    max_depth: usize,
    max_entries: u64,
}

#[derive(Debug, Default)]
struct ScanAccumulator {
    size_bytes: u64,
    file_count: u64,
    directory_count: u64,
    visited_entries: u64,
    is_partial: bool,
}

/// Run the directory usage scan once.
///
/// The preferred backend is `dust` because it is optimized for disk usage
/// trees. If it is unavailable, times out, or returns unparsable output, the
/// bounded Rust scanner provides a deterministic fallback.
pub async fn collect_directory_usage_once() -> Vec<DirectoryUsageInfo> {
    if let Some(entries) = collect_with_dust(default_scan_roots()).await {
        return entries;
    }

    tokio::task::spawn_blocking(|| collect_directory_usage_from_paths(default_scan_roots()))
        .await
        .unwrap_or_default()
}

fn default_scan_roots() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };

    [
        "Desktop",
        "Documents",
        "Downloads",
        "Movies",
        "Music",
        "Pictures",
    ]
    .into_iter()
    .map(|name| home.join(name))
    .filter(|path| path.is_dir())
    .collect()
}

fn collect_directory_usage_from_paths(paths: Vec<PathBuf>) -> Vec<DirectoryUsageInfo> {
    let limits = ScanLimits {
        max_depth: MAX_SCAN_DEPTH,
        max_entries: MAX_SCAN_ENTRIES,
    };
    let mut entries = paths
        .into_iter()
        .filter_map(|path| scan_directory_usage(&path, limits).ok())
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| right.size_bytes.cmp(&left.size_bytes));
    entries.truncate(MAX_TOP_DIRECTORIES);
    entries
}

async fn collect_with_dust(paths: Vec<PathBuf>) -> Option<Vec<DirectoryUsageInfo>> {
    if paths.is_empty() {
        return None;
    }

    let mut command = tokio::process::Command::new("dust");
    command.args(["-d", "1", "-n", "8", "--no-colors"]);
    for path in paths {
        command.arg(path);
    }

    let output = tokio::time::timeout(Duration::from_secs(20), command.output())
        .await
        .ok()?
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let entries = parse_dust_output(&text);
    (!entries.is_empty()).then_some(entries)
}

fn parse_dust_output(output: &str) -> Vec<DirectoryUsageInfo> {
    let mut entries = output
        .lines()
        .filter_map(parse_dust_line)
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| right.size_bytes.cmp(&left.size_bytes));
    entries.truncate(MAX_TOP_DIRECTORIES);
    entries
}

fn parse_dust_line(line: &str) -> Option<DirectoryUsageInfo> {
    let clean = line.trim();
    if clean.is_empty() {
        return None;
    }

    let size_token = clean
        .split_whitespace()
        .find(|token| parse_size_token(token).is_some())?;
    let size_bytes = parse_size_token(size_token)?;
    let path_start = clean.find('/').or_else(|| clean.find('~'))?;
    let path = clean[path_start..]
        .trim()
        .trim_matches(|ch: char| matches!(ch, '│' | '├' | '└' | '─' | '┌'))
        .trim()
        .to_string();
    if path.is_empty() {
        return None;
    }

    Some(DirectoryUsageInfo {
        path,
        size_bytes,
        file_count: 0,
        directory_count: 0,
        is_partial: false,
    })
}

fn parse_size_token(token: &str) -> Option<u64> {
    let normalized = token.trim().trim_end_matches('B');
    let split_at = normalized
        .char_indices()
        .find(|(_, ch)| !(ch.is_ascii_digit() || *ch == '.'))
        .map(|(index, _)| index)
        .unwrap_or(normalized.len());
    if split_at == 0 {
        return None;
    }

    let value = normalized[..split_at].parse::<f64>().ok()?;
    let unit = normalized[split_at..].to_ascii_lowercase();
    let multiplier = match unit.as_str() {
        "" | "b" => 1.0,
        "k" | "kb" | "ki" | "kib" => 1024.0,
        "m" | "mb" | "mi" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gi" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "ti" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    Some((value * multiplier) as u64)
}

fn scan_directory_usage(
    root: &Path,
    limits: ScanLimits,
) -> Result<DirectoryUsageInfo, std::io::Error> {
    let mut acc = ScanAccumulator::default();
    let mut queue = VecDeque::from([(root.to_path_buf(), 0usize)]);

    while let Some((path, depth)) = queue.pop_front() {
        if acc.visited_entries >= limits.max_entries {
            acc.is_partial = true;
            break;
        }

        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => {
                acc.is_partial = true;
                continue;
            }
        };

        acc.visited_entries += 1;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_file() {
            acc.file_count += 1;
            acc.size_bytes = acc.size_bytes.saturating_add(metadata.len());
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }

        acc.directory_count += 1;
        if depth >= limits.max_depth {
            acc.is_partial = true;
            continue;
        }

        let children = match fs::read_dir(&path) {
            Ok(children) => children,
            Err(_) => {
                acc.is_partial = true;
                continue;
            }
        };
        for child in children.flatten() {
            queue.push_back((child.path(), depth + 1));
        }
    }

    Ok(DirectoryUsageInfo {
        path: root.to_string_lossy().to_string(),
        size_bytes: acc.size_bytes,
        file_count: acc.file_count,
        directory_count: acc.directory_count,
        is_partial: acc.is_partial,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_file(path: &Path, bytes: usize) {
        let mut file = fs::File::create(path).expect("test file should be creatable");
        file.write_all(&vec![b'x'; bytes])
            .expect("test file should be writable");
    }

    /// Objective: Verify recursive size accounting across nested directories.
    /// Invariants: File count and byte total include child files below root.
    #[test]
    fn test_scan_directory_usage_counts_nested_files() {
        let temp = tempfile::tempdir().expect("tempdir should be available");
        let nested = temp.path().join("nested");
        fs::create_dir(&nested).expect("nested directory should be creatable");
        write_file(&temp.path().join("a.bin"), 128);
        write_file(&nested.join("b.bin"), 256);

        let usage = scan_directory_usage(
            temp.path(),
            ScanLimits {
                max_depth: 4,
                max_entries: 100,
            },
        )
        .expect("scan should succeed");

        assert_eq!(
            usage.size_bytes, 384,
            "scanner should sum nested file sizes"
        );
        assert_eq!(usage.file_count, 2, "scanner should count nested files");
        assert!(
            usage.directory_count >= 2,
            "scanner should count root and nested directories"
        );
        assert!(!usage.is_partial, "complete scan should not be partial");
    }

    /// Objective: Verify scan limits stop traversal deterministically.
    /// Invariants: Partial flag is set once the entry budget is exhausted.
    #[test]
    fn test_scan_directory_usage_marks_partial_on_entry_limit() {
        let temp = tempfile::tempdir().expect("tempdir should be available");
        for index in 0..5 {
            write_file(&temp.path().join(format!("{}.bin", index)), 16);
        }

        let usage = scan_directory_usage(
            temp.path(),
            ScanLimits {
                max_depth: 4,
                max_entries: 2,
            },
        )
        .expect("limited scan should still return a sample");

        assert!(
            usage.is_partial,
            "entry-limited scan should be marked partial"
        );
        assert!(
            usage.file_count <= 2,
            "entry limit should cap the amount of counted files"
        );
    }

    /// Objective: Verify top-directory ordering is descending by size.
    /// Invariants: Larger directory samples appear before smaller ones.
    #[test]
    fn test_collect_directory_usage_from_paths_sorts_by_size() {
        let temp = tempfile::tempdir().expect("tempdir should be available");
        let small = temp.path().join("small");
        let large = temp.path().join("large");
        fs::create_dir(&small).expect("small directory should be creatable");
        fs::create_dir(&large).expect("large directory should be creatable");
        write_file(&small.join("a.bin"), 64);
        write_file(&large.join("b.bin"), 512);

        let entries = collect_directory_usage_from_paths(vec![small, large]);

        assert_eq!(entries.len(), 2, "both test directories should be returned");
        assert!(
            entries[0].size_bytes >= entries[1].size_bytes,
            "directory samples should be sorted by descending size"
        );
    }

    /// Objective: Verify `dust` text output can be converted into samples.
    /// Invariants: Parsed entries keep paths and convert human units to bytes.
    #[test]
    fn test_parse_dust_output_extracts_paths_and_sizes() {
        let output = "  1.5G ┌── /Users/test/Downloads\n 32M ├── /Users/test/Documents\n";
        let entries = parse_dust_output(output);

        assert_eq!(entries.len(), 2, "two dust rows should be parsed");
        assert_eq!(
            entries[0].path, "/Users/test/Downloads",
            "largest parsed path should sort first"
        );
        assert_eq!(
            entries[0].size_bytes,
            (1.5 * 1024.0 * 1024.0 * 1024.0) as u64,
            "GiB dust unit should convert to bytes"
        );
    }

    /// Objective: Verify size parser accepts common `dust` unit spellings.
    /// Invariants: Binary unit suffixes are converted with 1024 multipliers.
    #[test]
    fn test_parse_size_token_supports_common_units() {
        assert_eq!(
            parse_size_token("512B"),
            Some(512),
            "bytes should parse directly"
        );
        assert_eq!(
            parse_size_token("1K"),
            Some(1024),
            "K should use binary scale"
        );
        assert_eq!(
            parse_size_token("2MiB"),
            Some(2 * 1024 * 1024),
            "MiB should use binary scale"
        );
        assert_eq!(
            parse_size_token("nope"),
            None,
            "non-size tokens should be ignored"
        );
    }
}
