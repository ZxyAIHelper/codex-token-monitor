use std::path::Path;

use chrono::{DateTime, SecondsFormat, Timelike, Utc};
use serde::Serialize;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

use crate::codex_log::TokenCountEvent;

#[derive(Clone)]
pub struct UsageStore {
    pool: SqlitePool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SessionSummary {
    pub session_id: String,
    pub path: String,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub tool_calls: i64,
    pub tool_output_bytes: i64,
    pub last_seen_at: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TimeBucket {
    pub bucket: String,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub session_count: i64,
}

impl UsageStore {
    pub async fn memory() -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        Ok(Self { pool })
    }

    pub async fn open(path: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::new()
            .filename(Path::new(path))
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            create table if not exists session_files (
              path text primary key,
              session_id text not null,
              file_size integer not null default 0,
              modified_at text not null default '',
              parsed_offset integer not null default 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            create table if not exists sessions (
              session_id text primary key,
              path text not null,
              total_tokens integer not null default 0,
              input_tokens integer not null default 0,
              cached_input_tokens integer not null default 0,
              output_tokens integer not null default 0,
              reasoning_output_tokens integer not null default 0,
              tool_calls integer not null default 0,
              tool_output_bytes integer not null default 0,
              last_seen_at text not null default ''
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            create table if not exists hourly_sessions (
              bucket text not null,
              session_id text not null,
              primary key (bucket, session_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            create table if not exists hourly_buckets (
              bucket text primary key,
              total_tokens integer not null default 0,
              input_tokens integer not null default 0,
              output_tokens integer not null default 0,
              session_count integer not null default 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_token_count(
        &self,
        session_id: &str,
        path: &str,
        event: TokenCountEvent,
    ) -> Result<(), sqlx::Error> {
        let bucket = hour_bucket(&event.timestamp)?;
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            insert into sessions (
              session_id, path, total_tokens, input_tokens, cached_input_tokens,
              output_tokens, reasoning_output_tokens, last_seen_at
            )
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            on conflict(session_id) do update set
              path = excluded.path,
              total_tokens = sessions.total_tokens + excluded.total_tokens,
              input_tokens = sessions.input_tokens + excluded.input_tokens,
              cached_input_tokens = sessions.cached_input_tokens + excluded.cached_input_tokens,
              output_tokens = sessions.output_tokens + excluded.output_tokens,
              reasoning_output_tokens = sessions.reasoning_output_tokens + excluded.reasoning_output_tokens,
              last_seen_at = excluded.last_seen_at
            "#,
        )
        .bind(session_id)
        .bind(path)
        .bind(event.total_tokens)
        .bind(event.input_tokens)
        .bind(event.cached_input_tokens)
        .bind(event.output_tokens)
        .bind(event.reasoning_output_tokens)
        .bind(&event.timestamp)
        .execute(&mut *tx)
        .await?;

        let session_delta = sqlx::query(
            r#"
            insert into hourly_sessions (bucket, session_id)
            values (?1, ?2)
            on conflict(bucket, session_id) do nothing
            "#,
        )
        .bind(&bucket)
        .bind(session_id)
        .execute(&mut *tx)
        .await?
        .rows_affected() as i64;

        sqlx::query(
            r#"
            insert into hourly_buckets (
              bucket, total_tokens, input_tokens, output_tokens, session_count
            )
            values (?1, ?2, ?3, ?4, ?5)
            on conflict(bucket) do update set
              total_tokens = hourly_buckets.total_tokens + excluded.total_tokens,
              input_tokens = hourly_buckets.input_tokens + excluded.input_tokens,
              output_tokens = hourly_buckets.output_tokens + excluded.output_tokens,
              session_count = hourly_buckets.session_count + excluded.session_count
            "#,
        )
        .bind(bucket)
        .bind(event.total_tokens)
        .bind(event.input_tokens)
        .bind(event.output_tokens)
        .bind(session_delta)
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    pub async fn sessions(&self) -> Result<Vec<SessionSummary>, sqlx::Error> {
        sqlx::query_as::<_, SessionSummary>(
            r#"
            select session_id, path, total_tokens, input_tokens, cached_input_tokens,
                   output_tokens, reasoning_output_tokens, tool_calls,
                   tool_output_bytes, last_seen_at
            from sessions
            order by total_tokens desc
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn hourly_totals(&self) -> Result<Vec<TimeBucket>, sqlx::Error> {
        sqlx::query_as::<_, TimeBucket>(
            r#"
            select bucket, total_tokens, input_tokens, output_tokens, session_count
            from hourly_buckets
            order by bucket asc
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }
}

fn hour_bucket(timestamp: &str) -> Result<String, sqlx::Error> {
    let parsed = DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| sqlx::Error::Protocol(format!("invalid token timestamp: {err}")))?;

    Ok(parsed
        .with_minute(0)
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .expect("zeroed hour timestamp should be valid")
        .to_rfc3339_opts(SecondsFormat::Secs, true))
}
