use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, SystemTime},
};

use chrono::{DateTime, SecondsFormat, Utc};
use serde::Deserialize;

use crate::{
    scanner::{default_codex_sessions_dir, extract_session_id, scan_file, should_scan_path},
    usage_store::UsageStore,
};

const RECONCILE_INTERVAL: Duration = Duration::from_secs(7);
const RECENT_WINDOW: Duration = Duration::from_secs(30 * 24 * 60 * 60);
const MAX_SCAN_FILES: usize = 500;

#[derive(Debug)]
struct Candidate {
    path: PathBuf,
    size: u64,
    modified: SystemTime,
    modified_at: String,
}

pub fn start_background_monitor(store: UsageStore, paused: Arc<AtomicBool>) {
    match thread::Builder::new()
        .name("codex-token-monitor".to_string())
        .spawn(move || monitor_loop(store, paused))
    {
        Ok(_) => {}
        Err(err) => eprintln!("codex-token-monitor: failed to start monitor thread: {err}"),
    }
}

fn monitor_loop(store: UsageStore, paused: Arc<AtomicBool>) {
    let Some(sessions_dir) = default_codex_sessions_dir() else {
        eprintln!("codex-token-monitor: home directory unavailable");
        return;
    };

    let mut logged_missing_sessions_dir = false;
    loop {
        if paused.load(Ordering::Relaxed) {
            thread::sleep(RECONCILE_INTERVAL);
            continue;
        }

        if !sessions_dir_ready(&sessions_dir) {
            if !logged_missing_sessions_dir {
                eprintln!(
                    "codex-token-monitor: waiting for sessions directory: {}",
                    sessions_dir.display()
                );
                logged_missing_sessions_dir = true;
            }
            thread::sleep(RECONCILE_INTERVAL);
            continue;
        }
        logged_missing_sessions_dir = false;

        if let Err(err) = reconcile_once(&store, &sessions_dir) {
            eprintln!("codex-token-monitor: reconciliation failed: {err}");
        }
        thread::sleep(RECONCILE_INTERVAL);
    }
}

fn sessions_dir_ready(sessions_dir: &Path) -> bool {
    sessions_dir.is_dir()
}

fn reconcile_once(store: &UsageStore, sessions_dir: &Path) -> Result<(), String> {
    sync_session_index(store, sessions_dir)?;

    let mut candidates = Vec::new();
    collect_recent_jsonl_files(sessions_dir, &mut candidates)?;
    candidates.sort_by(|left, right| {
        right
            .modified
            .cmp(&left.modified)
            .then_with(|| left.path.cmp(&right.path))
    });
    candidates.truncate(MAX_SCAN_FILES);

    let mut skipped = 0;
    for candidate in candidates {
        if let Err(err) = reconcile_file(store, &candidate) {
            skipped += 1;
            if skipped <= 3 {
                eprintln!(
                    "codex-token-monitor: scan skipped {}: {err}",
                    candidate.path.display()
                );
            }
        }
    }
    if skipped > 3 {
        eprintln!("codex-token-monitor: scan skipped {skipped} files this cycle");
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct SessionIndexLine {
    id: Option<String>,
    thread_name: Option<String>,
}

fn sync_session_index(store: &UsageStore, sessions_dir: &Path) -> Result<(), String> {
    let Some(codex_dir) = sessions_dir.parent() else {
        return Ok(());
    };
    let index_path = codex_dir.join("session_index.jsonl");
    if !index_path.is_file() {
        return Ok(());
    }

    let file = fs::File::open(&index_path)
        .map_err(|err| format!("open {} failed: {err}", index_path.display()))?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.map_err(|err| format!("read {} failed: {err}", index_path.display()))?;
        let Ok(entry) = serde_json::from_str::<SessionIndexLine>(&line) else {
            continue;
        };
        let session_id = entry.id.unwrap_or_default();
        let session_name = entry.thread_name.unwrap_or_default();
        tauri::async_runtime::block_on(store.record_session_name(&session_id, &session_name))
            .map_err(|err| err.to_string())?;
    }

    Ok(())
}

fn reconcile_file(store: &UsageStore, candidate: &Candidate) -> Result<(), String> {
    let path_text = candidate.path.to_string_lossy().to_string();
    let session_id = extract_session_id(&candidate.path)
        .ok_or_else(|| format!("invalid rollout filename: {}", candidate.path.display()))?;
    let stored = tauri::async_runtime::block_on(store.session_file_offset(&path_text))
        .map_err(|err| err.to_string())?;
    let stored_offset = stored
        .as_ref()
        .map(|offset| offset.parsed_offset.max(0) as u64)
        .unwrap_or(0);

    let start_offset = if stored_offset > candidate.size {
        eprintln!(
            "codex-token-monitor: file shrank, rescanning from start: {}",
            candidate.path.display()
        );
        0
    } else {
        stored_offset
    };

    if stored.is_some() && candidate.size <= start_offset {
        return Ok(());
    }

    let parsed_offset = scan_file(store, &candidate.path, start_offset)?;
    tauri::async_runtime::block_on(store.set_session_file_offset(
        &path_text,
        &session_id,
        candidate.size,
        &candidate.modified_at,
        parsed_offset,
    ))
    .map_err(|err| err.to_string())?;

    Ok(())
}

fn collect_recent_jsonl_files(dir: &Path, candidates: &mut Vec<Candidate>) -> Result<(), String> {
    let cutoff = SystemTime::now()
        .checked_sub(RECENT_WINDOW)
        .unwrap_or(SystemTime::UNIX_EPOCH);
    collect_recent_jsonl_files_inner(dir, candidates, cutoff)
}

fn collect_recent_jsonl_files_inner(
    dir: &Path,
    candidates: &mut Vec<Candidate>,
    cutoff: SystemTime,
) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|err| format!("read_dir failed for {}: {err}", dir.display()))?;

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("codex-token-monitor: directory entry skipped: {err}");
                continue;
            }
        };
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                eprintln!(
                    "codex-token-monitor: metadata skipped for {}: {err}",
                    path.display()
                );
                continue;
            }
        };

        if metadata.is_dir() {
            if let Err(err) = collect_recent_jsonl_files_inner(&path, candidates, cutoff) {
                eprintln!("codex-token-monitor: traversal skipped: {err}");
            }
            continue;
        }

        if !metadata.is_file() || !should_scan_path(&path) {
            continue;
        }

        let modified = match metadata.modified() {
            Ok(modified) => modified,
            Err(err) => {
                eprintln!(
                    "codex-token-monitor: modified time skipped for {}: {err}",
                    path.display()
                );
                continue;
            }
        };
        if modified < cutoff {
            continue;
        }

        candidates.push(Candidate {
            path,
            size: metadata.len(),
            modified,
            modified_at: system_time_to_rfc3339(modified),
        });
    }

    Ok(())
}

fn system_time_to_rfc3339(time: SystemTime) -> String {
    DateTime::<Utc>::from(time).to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sessions_dir_ready_tracks_directory_creation() {
        let dir = std::env::temp_dir().join(format!(
            "codex-token-monitor-missing-sessions-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        assert!(!sessions_dir_ready(&dir));

        std::fs::create_dir_all(&dir).unwrap();
        assert!(sessions_dir_ready(&dir));

        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn sync_session_index_records_thread_names() {
        let store = tauri::async_runtime::block_on(async {
            let store = UsageStore::memory().await.unwrap();
            store.init().await.unwrap();
            store
                .record_token_count(
                    "019f08f1-c13e-7ea1-b57d-8ba3bc9d4186",
                    "C:/tmp/session.jsonl",
                    crate::codex_log::TokenCountEvent {
                        timestamp: "2026-06-27T12:34:56Z".to_string(),
                        input_tokens: 10,
                        cached_input_tokens: 0,
                        output_tokens: 2,
                        reasoning_output_tokens: 0,
                        total_tokens: 12,
                    },
                )
                .await
                .unwrap();
            store
        });
        let codex_dir = std::env::temp_dir().join(format!(
            "codex-token-monitor-index-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let sessions_dir = codex_dir.join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();
        std::fs::write(
            codex_dir.join("session_index.jsonl"),
            r#"{"id":"019f08f1-c13e-7ea1-b57d-8ba3bc9d4186","thread_name":"Token 监控面板","updated_at":"2026-06-28T08:00:00Z"}"#,
        )
        .unwrap();

        sync_session_index(&store, &sessions_dir).unwrap();

        tauri::async_runtime::block_on(async {
            let sessions = store.sessions().await.unwrap();
            assert_eq!(sessions[0].session_name, "Token 监控面板");
        });

        let _ = std::fs::remove_dir_all(&codex_dir);
    }
}
