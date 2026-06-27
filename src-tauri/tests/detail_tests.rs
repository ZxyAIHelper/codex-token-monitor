use codex_token_monitor_lib::codex_log::TokenCountEvent;
use codex_token_monitor_lib::usage_store::UsageStore;

#[tokio::test]
async fn detail_tests_returns_session_turns_in_timestamp_order() {
    let store = UsageStore::memory().await.unwrap();
    store.init().await.unwrap();

    store
        .record_token_count(
            "session-a",
            "C:/tmp/session-a.jsonl",
            token_event("2026-06-27T12:20:00Z", 50, 40, 5, 2),
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-a",
            "C:/tmp/session-a.jsonl",
            token_event("2026-06-27T12:10:00Z", 100, 70, 20, 4),
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-b",
            "C:/tmp/session-b.jsonl",
            token_event("2026-06-27T12:15:00Z", 999, 900, 90, 9),
        )
        .await
        .unwrap();

    let turns = store.session_turns("session-a").await.unwrap();

    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].timestamp, "2026-06-27T12:10:00Z");
    assert_eq!(turns[0].total_tokens, 100);
    assert_eq!(turns[0].input_tokens, 70);
    assert_eq!(turns[0].cached_input_tokens, 20);
    assert_eq!(turns[0].output_tokens, 30);
    assert_eq!(turns[0].reasoning_output_tokens, 4);
    assert_eq!(turns[1].timestamp, "2026-06-27T12:20:00Z");
    assert_eq!(turns[1].total_tokens, 50);
}

#[tokio::test]
async fn detail_tests_daily_totals_use_utc_day_and_distinct_sessions() {
    let store = UsageStore::memory().await.unwrap();
    store.init().await.unwrap();

    store
        .record_token_count(
            "session-a",
            "C:/tmp/session-a.jsonl",
            token_event("2026-06-27T23:50:00-02:00", 100, 80, 10, 1),
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-a",
            "C:/tmp/session-a.jsonl",
            token_event("2026-06-28T02:10:00Z", 50, 40, 5, 2),
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-b",
            "C:/tmp/session-b.jsonl",
            token_event("2026-06-28T04:00:00Z", 25, 20, 0, 0),
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-c",
            "C:/tmp/session-c.jsonl",
            token_event("2026-06-27T23:00:00Z", 10, 7, 1, 0),
        )
        .await
        .unwrap();

    let days = store.daily_totals().await.unwrap();

    assert_eq!(days.len(), 2);
    assert_eq!(days[0].bucket, "2026-06-27");
    assert_eq!(days[0].total_tokens, 10);
    assert_eq!(days[0].input_tokens, 7);
    assert_eq!(days[0].output_tokens, 3);
    assert_eq!(days[0].session_count, 1);
    assert_eq!(days[1].bucket, "2026-06-28");
    assert_eq!(days[1].total_tokens, 175);
    assert_eq!(days[1].input_tokens, 140);
    assert_eq!(days[1].output_tokens, 35);
    assert_eq!(days[1].session_count, 2);
}

fn token_event(
    timestamp: &str,
    total_tokens: i64,
    input_tokens: i64,
    cached_input_tokens: i64,
    reasoning_output_tokens: i64,
) -> TokenCountEvent {
    TokenCountEvent {
        timestamp: timestamp.to_string(),
        input_tokens,
        cached_input_tokens,
        output_tokens: total_tokens - input_tokens,
        reasoning_output_tokens,
        total_tokens,
    }
}
