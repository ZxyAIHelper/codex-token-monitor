use codex_token_monitor_lib::alerts::{
    token_level, AlertLevel, HOURLY_CRITICAL_TOKENS, HOURLY_WARNING_TOKENS,
};
use codex_token_monitor_lib::scanner::{extract_session_id, should_scan_path};
use std::path::Path;

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
        Some(AlertLevel::Warning)
    );
    assert_eq!(
        token_level(
            HOURLY_CRITICAL_TOKENS,
            HOURLY_WARNING_TOKENS,
            HOURLY_CRITICAL_TOKENS
        ),
        Some(AlertLevel::Critical)
    );
}
