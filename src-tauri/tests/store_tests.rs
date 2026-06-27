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
