use std::{
    fs,
    path::{Path, PathBuf},
    thread,
};

use crate::{
    scanner::{default_codex_sessions_dir, scan_file, should_scan_path},
    usage_store::UsageStore,
};

const INITIAL_SCAN_FILE_LIMIT: usize = 200;

pub fn start_background_monitor(store: UsageStore) {
    let _ = thread::Builder::new()
        .name("codex-token-monitor".to_string())
        .spawn(move || {
            let Some(sessions_dir) = default_codex_sessions_dir() else {
                eprintln!("codex-token-monitor: home directory unavailable");
                return;
            };

            if !sessions_dir.is_dir() {
                eprintln!("codex-token-monitor: sessions directory not found");
                return;
            }

            let mut paths = Vec::new();
            collect_jsonl_files(&sessions_dir, &mut paths, INITIAL_SCAN_FILE_LIMIT);
            let mut skipped = 0;
            for path in paths {
                if let Err(err) = scan_file(&store, &path, 0) {
                    skipped += 1;
                    if skipped == 1 {
                        eprintln!("codex-token-monitor: scan skipped a file: {err}");
                    }
                }
            }
            if skipped > 1 {
                eprintln!("codex-token-monitor: scan skipped {skipped} files");
            }
        });
}

fn collect_jsonl_files(dir: &Path, paths: &mut Vec<PathBuf>, limit: usize) {
    if paths.len() >= limit {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        if paths.len() >= limit {
            break;
        }

        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, paths, limit);
        } else if should_scan_path(&path) {
            paths.push(path);
        }
    }
}
