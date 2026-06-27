use chrono::{DateTime, Utc};
use codex_token_monitor_lib::codex_log::TokenCountEvent;
use codex_token_monitor_lib::usage_store::UsageStore;

#[tokio::test]
async fn store_tests_stores_session_and_hourly_totals() {
    let store = UsageStore::memory().await.unwrap();
    store.init().await.unwrap();

    store
        .record_token_count(
            "session-a",
            "C:/tmp/session.jsonl",
            TokenCountEvent {
                timestamp: "2026-06-27T12:34:56Z".to_string(),
                input_tokens: 100,
                cached_input_tokens: 40,
                output_tokens: 7,
                reasoning_output_tokens: 3,
                total_tokens: 107,
            },
        )
        .await
        .unwrap();

    let sessions = store.sessions().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, "session-a");
    assert_eq!(sessions[0].total_tokens, 107);
    assert_eq!(sessions[0].input_tokens, 100);
    assert_eq!(sessions[0].cached_input_tokens, 40);
    assert_eq!(sessions[0].output_tokens, 7);
    assert_eq!(sessions[0].reasoning_output_tokens, 3);

    let hours = store.hourly_totals().await.unwrap();
    assert_eq!(hours.len(), 1);
    assert_eq!(hours[0].bucket, "2026-06-27T12:00:00Z");
    assert_eq!(hours[0].total_tokens, 107);
    assert_eq!(hours[0].input_tokens, 100);
    assert_eq!(hours[0].output_tokens, 7);
    assert_eq!(hours[0].session_count, 1);
}

#[tokio::test]
async fn store_tests_rejects_invalid_timestamp_without_writing_aggregates() {
    let store = UsageStore::memory().await.unwrap();
    store.init().await.unwrap();

    let err = store
        .record_token_count(
            "session-a",
            "C:/tmp/session.jsonl",
            token_event("not-a-timestamp", 100),
        )
        .await
        .unwrap_err();

    assert!(err.to_string().contains("invalid token timestamp"));
    assert!(store.sessions().await.unwrap().is_empty());
    assert!(store.hourly_totals().await.unwrap().is_empty());
}

#[tokio::test]
async fn store_tests_counts_distinct_sessions_per_hour() {
    let store = UsageStore::memory().await.unwrap();
    store.init().await.unwrap();

    store
        .record_token_count(
            "session-a",
            "C:/tmp/session-a.jsonl",
            token_event("2026-06-27T12:10:00Z", 100),
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-a",
            "C:/tmp/session-a.jsonl",
            token_event("2026-06-27T12:20:00Z", 50),
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-b",
            "C:/tmp/session-b.jsonl",
            token_event("2026-06-27T12:30:00Z", 25),
        )
        .await
        .unwrap();

    let hours = store.hourly_totals().await.unwrap();
    assert_eq!(hours.len(), 1);
    assert_eq!(hours[0].bucket, "2026-06-27T12:00:00Z");
    assert_eq!(hours[0].total_tokens, 175);
    assert_eq!(hours[0].session_count, 2);
}

#[tokio::test]
async fn store_tests_dashboard_summary_uses_inserted_token_events() {
    let store = UsageStore::memory().await.unwrap();
    store.init().await.unwrap();
    let now = DateTime::parse_from_rfc3339("2026-06-27T12:45:00Z")
        .unwrap()
        .with_timezone(&Utc);

    store
        .record_token_count(
            "session-a",
            "C:/tmp/session-a.jsonl",
            TokenCountEvent {
                timestamp: "2026-06-27T12:10:00Z".to_string(),
                input_tokens: 100,
                cached_input_tokens: 20,
                output_tokens: 30,
                reasoning_output_tokens: 5,
                total_tokens: 130,
            },
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-b",
            "C:/tmp/session-b.jsonl",
            TokenCountEvent {
                timestamp: "2026-06-27T08:15:00Z".to_string(),
                input_tokens: 40,
                cached_input_tokens: 0,
                output_tokens: 10,
                reasoning_output_tokens: 0,
                total_tokens: 50,
            },
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-old",
            "C:/tmp/session-old.jsonl",
            TokenCountEvent {
                timestamp: "2026-06-26T23:15:00Z".to_string(),
                input_tokens: 999,
                cached_input_tokens: 0,
                output_tokens: 1,
                reasoning_output_tokens: 0,
                total_tokens: 1000,
            },
        )
        .await
        .unwrap();

    let summary = store.dashboard_summary_at(now).await.unwrap();

    assert_eq!(summary.today_total_tokens, 180);
    assert_eq!(summary.last_hour_tokens, 130);
    assert_eq!(summary.last_five_hours_tokens, 180);
    assert_eq!(summary.active_session_count, 2);
    assert_eq!(summary.input_tokens, 1139);
    assert_eq!(summary.output_tokens, 41);
}

fn token_event(timestamp: &str, total_tokens: i64) -> TokenCountEvent {
    TokenCountEvent {
        timestamp: timestamp.to_string(),
        input_tokens: total_tokens,
        cached_input_tokens: 0,
        output_tokens: 0,
        reasoning_output_tokens: 0,
        total_tokens,
    }
}
