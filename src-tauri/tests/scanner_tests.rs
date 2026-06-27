use codex_token_monitor_lib::alerts::{
    token_level, AlertLevel, HOURLY_CRITICAL_TOKENS, HOURLY_WARNING_TOKENS,
};
use codex_token_monitor_lib::scanner::{extract_session_id, scan_file, should_scan_path};
use codex_token_monitor_lib::usage_store::UsageStore;
use std::io::Write;
use std::path::{Path, PathBuf};

#[test]
fn extracts_session_id_from_rollout_filename() {
    let path =
        Path::new("C:/tmp/rollout-2026-06-27T19-58-09-019f08f1-c13e-7ea1-b57d-8ba3bc9d4186.jsonl");

    assert_eq!(
        extract_session_id(path),
        Some("019f08f1-c13e-7ea1-b57d-8ba3bc9d4186".to_string())
    );
}

#[test]
fn rejects_non_jsonl_and_malformed_rollout_filenames() {
    let session_id = "019f08f1-c13e-7ea1-b57d-8ba3bc9d4186";

    assert_eq!(
        extract_session_id(Path::new(&format!(
            "C:/tmp/rollout-2026-06-27T19-58-09-{session_id}.txt"
        ))),
        None
    );
    assert_eq!(
        extract_session_id(Path::new(&format!("C:/tmp/rollout-{session_id}.jsonl"))),
        None
    );
    assert_eq!(
        extract_session_id(Path::new(&format!(
            "C:/tmp/rollout-not-a-date-{session_id}.jsonl"
        ))),
        None
    );
}

#[test]
fn scans_only_jsonl_files_case_insensitively() {
    assert!(should_scan_path(Path::new("a.jsonl")));
    assert!(should_scan_path(Path::new("a.JSONL")));
    assert!(!should_scan_path(Path::new("a.sqlite")));
    assert!(!should_scan_path(Path::new("README.md")));
    assert!(!should_scan_path(Path::new("jsonl")));
}

#[test]
fn classifies_alert_threshold_levels() {
    assert_eq!(
        token_level(
            HOURLY_WARNING_TOKENS - 1,
            HOURLY_WARNING_TOKENS,
            HOURLY_CRITICAL_TOKENS
        ),
        None
    );
    assert_eq!(
        token_level(
            HOURLY_WARNING_TOKENS,
            HOURLY_WARNING_TOKENS,
            HOURLY_CRITICAL_TOKENS
        ),
        None
    );
    assert_eq!(
        token_level(
            HOURLY_CRITICAL_TOKENS,
            HOURLY_WARNING_TOKENS,
            HOURLY_CRITICAL_TOKENS
        ),
        Some(AlertLevel::Warning)
    );
    assert_eq!(
        token_level(
            HOURLY_WARNING_TOKENS + 1,
            HOURLY_WARNING_TOKENS,
            HOURLY_CRITICAL_TOKENS
        ),
        Some(AlertLevel::Warning)
    );
    assert_eq!(
        token_level(
            HOURLY_CRITICAL_TOKENS + 1,
            HOURLY_WARNING_TOKENS,
            HOURLY_CRITICAL_TOKENS
        ),
        Some(AlertLevel::Critical)
    );
}

#[test]
fn scan_file_records_token_counts_and_returns_offsets() {
    let store = tauri::async_runtime::block_on(async {
        let store = UsageStore::memory().await.unwrap();
        store.init().await.unwrap();
        store
    });
    let path = temp_rollout_path("scan-offset");
    let mut file = std::fs::File::create(&path).unwrap();
    writeln!(
        file,
        r#"{{"timestamp":"2026-06-27T12:00:00Z","type":"event_msg","payload":{{"type":"token_count","info":{{"last_token_usage":{{"input_tokens":100,"cached_input_tokens":40,"output_tokens":7,"reasoning_output_tokens":3,"total_tokens":107}}}}}}}}"#
    )
    .unwrap();
    writeln!(
        file,
        r#"{{"timestamp":"2026-06-27T12:00:01Z","type":"response_item","payload":{{"type":"message","role":"assistant","content":[]}}}}"#
    )
    .unwrap();
    let first_len = file.metadata().unwrap().len();

    let first_offset = scan_file(&store, &path, 0).unwrap();

    assert_eq!(first_offset, first_len);
    tauri::async_runtime::block_on(async {
        let sessions = store.sessions().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(
            sessions[0].session_id,
            "019f08f1-c13e-7ea1-b57d-8ba3bc9d4186"
        );
        assert_eq!(sessions[0].total_tokens, 107);
    });

    writeln!(
        file,
        r#"{{"timestamp":"2026-06-27T13:00:00Z","type":"event_msg","payload":{{"type":"token_count","info":{{"last_token_usage":{{"input_tokens":50,"cached_input_tokens":10,"output_tokens":5,"reasoning_output_tokens":1,"total_tokens":55}}}}}}}}"#
    )
    .unwrap();

    let second_offset = scan_file(&store, &path, first_offset).unwrap();

    assert_eq!(second_offset, file.metadata().unwrap().len());
    tauri::async_runtime::block_on(async {
        let sessions = store.sessions().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_tokens, 162);
        let turns = store
            .session_turns("019f08f1-c13e-7ea1-b57d-8ba3bc9d4186")
            .await
            .unwrap();
        assert_eq!(turns.len(), 2);
    });

    cleanup_temp_rollout(&path);
}

#[test]
fn scan_file_records_tool_outputs() {
    let store = tauri::async_runtime::block_on(async {
        let store = UsageStore::memory().await.unwrap();
        store.init().await.unwrap();
        store
    });
    let path = temp_rollout_path("scan-tool-output");
    let mut file = std::fs::File::create(&path).unwrap();
    writeln!(
        file,
        r#"{{"timestamp":"2026-06-27T12:00:00Z","type":"response_item","payload":{{"type":"function_call_output","call_id":"call_1","output":"{}"}}}}"#,
        "x".repeat(128)
    )
    .unwrap();

    let offset = scan_file(&store, &path, 0).unwrap();

    assert_eq!(offset, file.metadata().unwrap().len());
    tauri::async_runtime::block_on(async {
        let sessions = store.sessions().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].tool_calls, 1);
        assert_eq!(sessions[0].tool_output_bytes, 128);
    });

    cleanup_temp_rollout(&path);
}

#[test]
fn scan_file_does_not_double_count_same_tool_line_when_rescanned() {
    let store = tauri::async_runtime::block_on(async {
        let store = UsageStore::memory().await.unwrap();
        store.init().await.unwrap();
        store
    });
    let path = temp_rollout_path("scan-tool-output-rescan");
    let mut file = std::fs::File::create(&path).unwrap();
    writeln!(
        file,
        r#"{{"timestamp":"2026-06-27T12:00:00Z","type":"response_item","payload":{{"type":"function_call_output","call_id":"call_1","output":"{}"}}}}"#,
        "x".repeat(128)
    )
    .unwrap();

    let first_offset = scan_file(&store, &path, 0).unwrap();
    let second_offset = scan_file(&store, &path, 0).unwrap();

    assert_eq!(first_offset, file.metadata().unwrap().len());
    assert_eq!(second_offset, file.metadata().unwrap().len());
    tauri::async_runtime::block_on(async {
        let sessions = store.sessions().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].tool_calls, 1);
        assert_eq!(sessions[0].tool_output_bytes, 128);
    });

    cleanup_temp_rollout(&path);
}

#[test]
fn scan_file_counts_tool_outputs_with_same_timestamp_and_size_on_different_lines() {
    let store = tauri::async_runtime::block_on(async {
        let store = UsageStore::memory().await.unwrap();
        store.init().await.unwrap();
        store
    });
    let path = temp_rollout_path("scan-tool-output-same-second");
    let mut file = std::fs::File::create(&path).unwrap();
    for call_id in ["call_1", "call_2"] {
        writeln!(
            file,
            r#"{{"timestamp":"2026-06-27T12:00:00.999Z","type":"response_item","payload":{{"type":"function_call_output","call_id":"{call_id}","output":"{}"}}}}"#,
            "x".repeat(128)
        )
        .unwrap();
    }

    let offset = scan_file(&store, &path, 0).unwrap();

    assert_eq!(offset, file.metadata().unwrap().len());
    tauri::async_runtime::block_on(async {
        let sessions = store.sessions().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].tool_calls, 2);
        assert_eq!(sessions[0].tool_output_bytes, 256);
    });

    cleanup_temp_rollout(&path);
}

#[test]
fn scan_file_skips_invalid_json_and_continues() {
    let store = tauri::async_runtime::block_on(async {
        let store = UsageStore::memory().await.unwrap();
        store.init().await.unwrap();
        store
    });
    let path = temp_rollout_path("scan-invalid-json");
    let mut file = std::fs::File::create(&path).unwrap();
    writeln!(file, r#"{{"timestamp":"2026-06-27T12:00:00Z""#).unwrap();
    writeln!(
        file,
        r#"{{"timestamp":"2026-06-27T12:00:01Z","type":"event_msg","payload":{{"type":"token_count","info":{{"last_token_usage":{{"input_tokens":10,"cached_input_tokens":0,"output_tokens":2,"reasoning_output_tokens":0,"total_tokens":12}}}}}}}}"#
    )
    .unwrap();

    let offset = scan_file(&store, &path, 0).unwrap();

    assert_eq!(offset, file.metadata().unwrap().len());
    tauri::async_runtime::block_on(async {
        let sessions = store.sessions().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_tokens, 12);
    });

    cleanup_temp_rollout(&path);
}

#[test]
fn scan_file_retries_partial_trailing_invalid_json() {
    let store = tauri::async_runtime::block_on(async {
        let store = UsageStore::memory().await.unwrap();
        store.init().await.unwrap();
        store
    });
    let path = temp_rollout_path("scan-partial-json");
    let mut file = std::fs::File::create(&path).unwrap();
    writeln!(
        file,
        r#"{{"timestamp":"2026-06-27T12:00:00Z","type":"event_msg","payload":{{"type":"token_count","info":{{"last_token_usage":{{"input_tokens":10,"cached_input_tokens":0,"output_tokens":2,"reasoning_output_tokens":0,"total_tokens":12}}}}}}}}"#
    )
    .unwrap();
    let complete_offset = file.metadata().unwrap().len();
    write!(file, r#"{{"timestamp":"2026-06-27T12:00:01Z""#).unwrap();

    let offset = scan_file(&store, &path, 0).unwrap();

    assert_eq!(offset, complete_offset);
    tauri::async_runtime::block_on(async {
        let sessions = store.sessions().await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].total_tokens, 12);
    });

    cleanup_temp_rollout(&path);
}

#[test]
fn usage_store_persists_session_file_offsets() {
    tauri::async_runtime::block_on(async {
        let store = UsageStore::memory().await.unwrap();
        store.init().await.unwrap();

        store
            .set_session_file_offset(
                "C:/tmp/session-a.jsonl",
                "session-a",
                2048,
                "2026-06-27T12:00:00Z",
                1024,
            )
            .await
            .unwrap();

        let offset = store
            .session_file_offset("C:/tmp/session-a.jsonl")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(offset.session_id, "session-a");
        assert_eq!(offset.file_size, 2048);
        assert_eq!(offset.modified_at, "2026-06-27T12:00:00Z");
        assert_eq!(offset.parsed_offset, 1024);
    });
}

fn temp_rollout_path(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "codex-token-monitor-{label}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("rollout-2026-06-27T19-58-09-019f08f1-c13e-7ea1-b57d-8ba3bc9d4186.jsonl")
}

fn cleanup_temp_rollout(path: &Path) {
    let _ = std::fs::remove_file(path);
    if let Some(parent) = path.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}
