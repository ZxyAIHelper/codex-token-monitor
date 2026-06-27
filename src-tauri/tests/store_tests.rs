use chrono::{DateTime, Utc};
use codex_token_monitor_lib::codex_log::TokenCountEvent;
use codex_token_monitor_lib::usage_store::UsageStore;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

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
async fn store_tests_duplicate_token_events_are_idempotent() {
    let store = UsageStore::memory().await.unwrap();
    store.init().await.unwrap();
    let now = DateTime::parse_from_rfc3339("2026-06-27T12:45:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = TokenCountEvent {
        timestamp: "2026-06-27T12:34:56+00:00".to_string(),
        input_tokens: 100,
        cached_input_tokens: 40,
        output_tokens: 7,
        reasoning_output_tokens: 3,
        total_tokens: 107,
    };

    store
        .record_token_count("session-a", "C:/tmp/session.jsonl", event.clone())
        .await
        .unwrap();
    store
        .record_token_count("session-a", "C:/tmp/session.jsonl", event)
        .await
        .unwrap();

    let sessions = store.sessions().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].total_tokens, 107);
    assert_eq!(sessions[0].input_tokens, 100);
    assert_eq!(sessions[0].cached_input_tokens, 40);
    assert_eq!(sessions[0].output_tokens, 7);
    assert_eq!(sessions[0].reasoning_output_tokens, 3);

    let turns = store.session_turns("session-a").await.unwrap();
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].timestamp, "2026-06-27T12:34:56Z");

    let hours = store.hourly_totals().await.unwrap();
    assert_eq!(hours.len(), 1);
    assert_eq!(hours[0].total_tokens, 107);
    assert_eq!(hours[0].session_count, 1);

    let days = store.daily_totals().await.unwrap();
    assert_eq!(days.len(), 1);
    assert_eq!(days[0].total_tokens, 107);
    assert_eq!(days[0].session_count, 1);

    let summary = store.dashboard_summary_at(now).await.unwrap();
    assert_eq!(summary.today_total_tokens, 107);
    assert_eq!(summary.last_hour_tokens, 107);
    assert_eq!(summary.last_five_hours_tokens, 107);
    assert_eq!(summary.active_session_count, 1);
    assert_eq!(summary.input_tokens, 100);
    assert_eq!(summary.output_tokens, 7);
}

#[tokio::test]
async fn store_tests_init_backfills_daily_totals_from_existing_token_events() {
    let db_path = std::env::temp_dir().join(format!(
        "codex-token-monitor-backfill-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_file(&db_path);

    {
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::query(
            r#"
            create table token_events (
              id integer primary key autoincrement,
              session_id text not null,
              path text not null,
              timestamp text not null,
              total_tokens integer not null default 0,
              input_tokens integer not null default 0,
              cached_input_tokens integer not null default 0,
              output_tokens integer not null default 0,
              reasoning_output_tokens integer not null default 0
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            r#"
            insert into token_events (
              session_id, path, timestamp, total_tokens, input_tokens,
              cached_input_tokens, output_tokens, reasoning_output_tokens
            )
            values
              ('session-a', 'C:/tmp/session-a.jsonl', '2026-06-27T23:50:00Z', 100, 80, 10, 20, 1),
              ('session-a', 'C:/tmp/session-a.jsonl', '2026-06-28T01:10:00Z', 50, 40, 5, 10, 2),
              ('session-b', 'C:/tmp/session-b.jsonl', '2026-06-28T02:00:00Z', 25, 20, 0, 5, 0)
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();
        pool.close().await;
    }

    {
        let store = UsageStore::open(db_path.to_str().unwrap()).await.unwrap();
        store.init().await.unwrap();

        let days = store.daily_totals().await.unwrap();
        assert_eq!(days.len(), 2);
        assert_eq!(days[0].bucket, "2026-06-27");
        assert_eq!(days[0].total_tokens, 100);
        assert_eq!(days[0].session_count, 1);
        assert_eq!(days[1].bucket, "2026-06-28");
        assert_eq!(days[1].total_tokens, 75);
        assert_eq!(days[1].input_tokens, 60);
        assert_eq!(days[1].output_tokens, 15);
        assert_eq!(days[1].session_count, 2);

        store
            .record_token_count(
                "session-a",
                "C:/tmp/session-a.jsonl",
                token_event("2026-06-28T03:00:00Z", 10),
            )
            .await
            .unwrap();

        let days = store.daily_totals().await.unwrap();
        assert_eq!(days[1].total_tokens, 85);
        assert_eq!(days[1].session_count, 2);
    }

    let _ = std::fs::remove_file(db_path);
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
    assert!(store.daily_totals().await.unwrap().is_empty());
    assert!(store.session_turns("session-a").await.unwrap().is_empty());
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
            "session-cross-hour",
            "C:/tmp/session-cross-hour.jsonl",
            TokenCountEvent {
                timestamp: "2026-06-27T11:50:00Z".to_string(),
                input_tokens: 60,
                cached_input_tokens: 0,
                output_tokens: 10,
                reasoning_output_tokens: 0,
                total_tokens: 70,
            },
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-before-hour",
            "C:/tmp/session-before-hour.jsonl",
            TokenCountEvent {
                timestamp: "2026-06-27T11:44:00Z".to_string(),
                input_tokens: 50,
                cached_input_tokens: 0,
                output_tokens: 10,
                reasoning_output_tokens: 0,
                total_tokens: 60,
            },
        )
        .await
        .unwrap();
    store
        .record_token_count(
            "session-five-hour",
            "C:/tmp/session-five-hour.jsonl",
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
            "session-before-five-hours",
            "C:/tmp/session-before-five-hours.jsonl",
            TokenCountEvent {
                timestamp: "2026-06-27T06:30:00Z".to_string(),
                input_tokens: 20,
                cached_input_tokens: 0,
                output_tokens: 5,
                reasoning_output_tokens: 0,
                total_tokens: 25,
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

    assert_eq!(summary.today_total_tokens, 335);
    assert_eq!(summary.last_hour_tokens, 200);
    assert_eq!(summary.last_five_hours_tokens, 310);
    assert_eq!(summary.active_session_count, 4);
    assert_eq!(summary.input_tokens, 1269);
    assert_eq!(summary.output_tokens, 66);
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
