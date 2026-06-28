use std::path::Path;

use chrono::{DateTime, Duration, SecondsFormat, Timelike, Utc};
use serde::Serialize;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

use crate::{
    alerts::{
        token_level, AlertItem, AlertKind, HOURLY_CRITICAL_TOKENS, HOURLY_WARNING_TOKENS,
        SESSION_CRITICAL_TOKENS, SESSION_WARNING_TOKENS, TOOL_OUTPUT_CRITICAL_BYTES,
        TOOL_OUTPUT_WARNING_BYTES,
    },
    codex_log::{TokenCountEvent, ToolOutputEvent},
};

#[derive(Clone)]
pub struct UsageStore {
    pool: SqlitePool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SessionSummary {
    pub session_id: String,
    pub session_name: String,
    pub cwd: String,
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

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TurnDetail {
    pub timestamp: String,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    pub today_total_tokens: i64,
    pub last_hour_tokens: i64,
    pub last_five_hours_tokens: i64,
    pub active_session_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SessionFileOffset {
    pub path: String,
    pub session_id: String,
    pub file_size: i64,
    pub modified_at: String,
    pub parsed_offset: i64,
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
            create table if not exists session_metadata (
              session_id text primary key,
              session_name text not null default '',
              cwd text not null default ''
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
            create table if not exists token_events (
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
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            delete from token_events
            where id not in (
              select min(id)
              from token_events
              group by session_id, path, timestamp, total_tokens, input_tokens,
                       cached_input_tokens, output_tokens, reasoning_output_tokens
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            create unique index if not exists idx_token_events_unique_event
            on token_events (
              session_id, path, timestamp, total_tokens, input_tokens,
              cached_input_tokens, output_tokens, reasoning_output_tokens
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            create table if not exists tool_events (
              id integer primary key autoincrement,
              session_id text not null,
              path text not null,
              line_start_offset integer,
              timestamp text not null,
              output_bytes integer not null default 0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        self.ensure_tool_event_offset_column().await?;

        sqlx::query(
            r#"
            delete from tool_events
            where line_start_offset is null
              and id not in (
              select min(id)
              from tool_events
              where line_start_offset is null
              group by session_id, path, timestamp, output_bytes
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            delete from tool_events
            where line_start_offset is not null
              and id not in (
              select min(id)
              from tool_events
              where line_start_offset is not null
              group by path, line_start_offset
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("drop index if exists idx_tool_events_unique_event")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            r#"
            create unique index if not exists idx_tool_events_unique_event
            on tool_events (path, line_start_offset)
            where line_start_offset is not null
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

        sqlx::query(
            r#"
            create table if not exists daily_sessions (
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
            create table if not exists daily_buckets (
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

        self.rebuild_aggregates_from_token_events().await?;

        Ok(())
    }

    pub async fn record_session_cwd(&self, session_id: &str, cwd: &str) -> Result<(), sqlx::Error> {
        if session_id.is_empty() || cwd.is_empty() {
            return Ok(());
        }

        sqlx::query(
            r#"
            insert into session_metadata (session_id, cwd)
            values (?1, ?2)
            on conflict(session_id) do update set
              cwd = excluded.cwd
            "#,
        )
        .bind(session_id)
        .bind(cwd)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn record_session_name(
        &self,
        session_id: &str,
        session_name: &str,
    ) -> Result<(), sqlx::Error> {
        if session_id.is_empty() || session_name.is_empty() {
            return Ok(());
        }

        sqlx::query(
            r#"
            insert into session_metadata (session_id, session_name)
            values (?1, ?2)
            on conflict(session_id) do update set
              session_name = excluded.session_name
            "#,
        )
        .bind(session_id)
        .bind(session_name)
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
        let event_timestamp = parse_timestamp_utc(&event.timestamp)?;
        let hour_bucket = hour_bucket_from_datetime(event_timestamp);
        let day_bucket = day_bucket_from_datetime(event_timestamp);
        let event_timestamp = event_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true);
        let mut tx = self.pool.begin().await?;

        let token_event_insert = sqlx::query(
            r#"
            insert into token_events (
              session_id, path, timestamp, total_tokens, input_tokens,
              cached_input_tokens, output_tokens, reasoning_output_tokens
            )
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            on conflict do nothing
            "#,
        )
        .bind(session_id)
        .bind(path)
        .bind(&event_timestamp)
        .bind(event.total_tokens)
        .bind(event.input_tokens)
        .bind(event.cached_input_tokens)
        .bind(event.output_tokens)
        .bind(event.reasoning_output_tokens)
        .execute(&mut *tx)
        .await?;
        if token_event_insert.rows_affected() == 0 {
            tx.commit().await?;
            return Ok(());
        }

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
              last_seen_at = case
                when sessions.last_seen_at > excluded.last_seen_at then sessions.last_seen_at
                else excluded.last_seen_at
              end
            "#,
        )
        .bind(session_id)
        .bind(path)
        .bind(event.total_tokens)
        .bind(event.input_tokens)
        .bind(event.cached_input_tokens)
        .bind(event.output_tokens)
        .bind(event.reasoning_output_tokens)
        .bind(&event_timestamp)
        .execute(&mut *tx)
        .await?;

        let session_delta = sqlx::query(
            r#"
            insert into hourly_sessions (bucket, session_id)
            values (?1, ?2)
            on conflict(bucket, session_id) do nothing
            "#,
        )
        .bind(&hour_bucket)
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
        .bind(hour_bucket)
        .bind(event.total_tokens)
        .bind(event.input_tokens)
        .bind(event.output_tokens)
        .bind(session_delta)
        .execute(&mut *tx)
        .await?;

        let daily_session_delta = sqlx::query(
            r#"
            insert into daily_sessions (bucket, session_id)
            values (?1, ?2)
            on conflict(bucket, session_id) do nothing
            "#,
        )
        .bind(&day_bucket)
        .bind(session_id)
        .execute(&mut *tx)
        .await?
        .rows_affected() as i64;

        sqlx::query(
            r#"
            insert into daily_buckets (
              bucket, total_tokens, input_tokens, output_tokens, session_count
            )
            values (?1, ?2, ?3, ?4, ?5)
            on conflict(bucket) do update set
              total_tokens = daily_buckets.total_tokens + excluded.total_tokens,
              input_tokens = daily_buckets.input_tokens + excluded.input_tokens,
              output_tokens = daily_buckets.output_tokens + excluded.output_tokens,
              session_count = daily_buckets.session_count + excluded.session_count
            "#,
        )
        .bind(day_bucket)
        .bind(event.total_tokens)
        .bind(event.input_tokens)
        .bind(event.output_tokens)
        .bind(daily_session_delta)
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    pub async fn record_tool_output(
        &self,
        session_id: &str,
        path: &str,
        line_start_offset: u64,
        event: ToolOutputEvent,
    ) -> Result<(), sqlx::Error> {
        let event_timestamp = parse_timestamp_utc(&event.timestamp)?;
        let event_timestamp = event_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true);
        let line_start_offset = i64::try_from(line_start_offset).map_err(|_| {
            sqlx::Error::Protocol(
                "tool output line offset exceeds sqlite integer range".to_string(),
            )
        })?;
        let mut tx = self.pool.begin().await?;

        let tool_event_insert = sqlx::query(
            r#"
            insert into tool_events (session_id, path, line_start_offset, timestamp, output_bytes)
            values (?1, ?2, ?3, ?4, ?5)
            on conflict do nothing
            "#,
        )
        .bind(session_id)
        .bind(path)
        .bind(line_start_offset)
        .bind(&event_timestamp)
        .bind(event.output_bytes)
        .execute(&mut *tx)
        .await?;
        if tool_event_insert.rows_affected() == 0 {
            tx.commit().await?;
            return Ok(());
        }

        sqlx::query(
            r#"
            insert into sessions (
              session_id, path, tool_calls, tool_output_bytes, last_seen_at
            )
            values (?1, ?2, 1, ?3, ?4)
            on conflict(session_id) do update set
              path = excluded.path,
              tool_calls = sessions.tool_calls + excluded.tool_calls,
              tool_output_bytes = sessions.tool_output_bytes + excluded.tool_output_bytes,
              last_seen_at = case
                when sessions.last_seen_at > excluded.last_seen_at then sessions.last_seen_at
                else excluded.last_seen_at
              end
            "#,
        )
        .bind(session_id)
        .bind(path)
        .bind(event.output_bytes)
        .bind(&event_timestamp)
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    async fn ensure_tool_event_offset_column(&self) -> Result<(), sqlx::Error> {
        let column_count = sqlx::query_scalar::<_, i64>(
            "select count(*) from pragma_table_info('tool_events') where name = 'line_start_offset'",
        )
        .fetch_one(&self.pool)
        .await?;

        if column_count == 0 {
            sqlx::query("alter table tool_events add column line_start_offset integer")
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub async fn session_file_offset(
        &self,
        path: &str,
    ) -> Result<Option<SessionFileOffset>, sqlx::Error> {
        sqlx::query_as::<_, SessionFileOffset>(
            r#"
            select path, session_id, file_size, modified_at, parsed_offset
            from session_files
            where path = ?1
            "#,
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn set_session_file_offset(
        &self,
        path: &str,
        session_id: &str,
        file_size: u64,
        modified_at: &str,
        parsed_offset: u64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            insert into session_files (path, session_id, file_size, modified_at, parsed_offset)
            values (?1, ?2, ?3, ?4, ?5)
            on conflict(path) do update set
              session_id = excluded.session_id,
              file_size = excluded.file_size,
              modified_at = excluded.modified_at,
              parsed_offset = excluded.parsed_offset
            "#,
        )
        .bind(path)
        .bind(session_id)
        .bind(file_size as i64)
        .bind(modified_at)
        .bind(parsed_offset as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn rebuild_aggregates_from_token_events(&self) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("delete from sessions")
            .execute(&mut *tx)
            .await?;
        sqlx::query("delete from hourly_sessions")
            .execute(&mut *tx)
            .await?;
        sqlx::query("delete from hourly_buckets")
            .execute(&mut *tx)
            .await?;
        sqlx::query("delete from daily_sessions")
            .execute(&mut *tx)
            .await?;
        sqlx::query("delete from daily_buckets")
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            insert into sessions (
              session_id, path, total_tokens, input_tokens, cached_input_tokens,
              output_tokens, reasoning_output_tokens, tool_calls,
              tool_output_bytes, last_seen_at
            )
            select
              all_sessions.session_id,
              (
                select latest.path
                from (
                  select session_id, path, timestamp, id from token_events
                  union all
                  select session_id, path, timestamp, id from tool_events
                ) latest
                where latest.session_id = all_sessions.session_id
                order by latest.timestamp desc, latest.id desc
                limit 1
              ),
              coalesce(token_totals.total_tokens, 0),
              coalesce(token_totals.input_tokens, 0),
              coalesce(token_totals.cached_input_tokens, 0),
              coalesce(token_totals.output_tokens, 0),
              coalesce(token_totals.reasoning_output_tokens, 0),
              coalesce(tool_totals.tool_calls, 0),
              coalesce(tool_totals.tool_output_bytes, 0),
              (
                select max(latest.timestamp)
                from (
                  select session_id, timestamp from token_events
                  union all
                  select session_id, timestamp from tool_events
                ) latest
                where latest.session_id = all_sessions.session_id
              )
            from (
              select session_id from token_events
              union
              select session_id from tool_events
            ) all_sessions
            left join (
              select
                session_id,
                coalesce(sum(total_tokens), 0) as total_tokens,
                coalesce(sum(input_tokens), 0) as input_tokens,
                coalesce(sum(cached_input_tokens), 0) as cached_input_tokens,
                coalesce(sum(output_tokens), 0) as output_tokens,
                coalesce(sum(reasoning_output_tokens), 0) as reasoning_output_tokens,
                max(timestamp) as last_seen_at
              from token_events
              group by session_id
            ) token_totals on token_totals.session_id = all_sessions.session_id
            left join (
              select
                session_id,
                count(*) as tool_calls,
                coalesce(sum(output_bytes), 0) as tool_output_bytes,
                max(timestamp) as last_seen_at
              from tool_events
              group by session_id
            ) tool_totals on tool_totals.session_id = all_sessions.session_id
            "#,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            insert into hourly_sessions (bucket, session_id)
            select strftime('%Y-%m-%dT%H:00:00Z', timestamp), session_id
            from token_events
            where strftime('%Y-%m-%dT%H:00:00Z', timestamp) is not null
            group by strftime('%Y-%m-%dT%H:00:00Z', timestamp), session_id
            "#,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            insert into hourly_buckets (
              bucket, total_tokens, input_tokens, output_tokens, session_count
            )
            select
              bucket,
              coalesce(sum(total_tokens), 0),
              coalesce(sum(input_tokens), 0),
              coalesce(sum(output_tokens), 0),
              count(distinct session_id)
            from (
              select
                strftime('%Y-%m-%dT%H:00:00Z', timestamp) as bucket,
                session_id,
                total_tokens,
                input_tokens,
                output_tokens
              from token_events
              where strftime('%Y-%m-%dT%H:00:00Z', timestamp) is not null
            ) hourly_events
            group by bucket
            "#,
        )
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            insert into daily_sessions (bucket, session_id)
            select date(timestamp), session_id
            from token_events
            where date(timestamp) is not null
            group by date(timestamp), session_id
            "#,
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            insert into daily_buckets (
              bucket, total_tokens, input_tokens, output_tokens, session_count
            )
            select
              date(timestamp),
              coalesce(sum(total_tokens), 0),
              coalesce(sum(input_tokens), 0),
              coalesce(sum(output_tokens), 0),
              count(distinct session_id)
            from token_events
            where date(timestamp) is not null
            group by date(timestamp)
            "#,
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }

    pub async fn sessions(&self) -> Result<Vec<SessionSummary>, sqlx::Error> {
        sqlx::query_as::<_, SessionSummary>(
            r#"
            select sessions.session_id,
                   coalesce(session_metadata.session_name, '') as session_name,
                   coalesce(session_metadata.cwd, '') as cwd,
                   path, total_tokens, input_tokens, cached_input_tokens,
                   output_tokens, reasoning_output_tokens, tool_calls,
                   tool_output_bytes, last_seen_at
            from sessions
            left join session_metadata on session_metadata.session_id = sessions.session_id
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

    pub async fn daily_totals(&self) -> Result<Vec<TimeBucket>, sqlx::Error> {
        sqlx::query_as::<_, TimeBucket>(
            r#"
            select bucket, total_tokens, input_tokens, output_tokens, session_count
            from daily_buckets
            order by bucket asc
            "#,
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn session_turns(&self, session_id: &str) -> Result<Vec<TurnDetail>, sqlx::Error> {
        sqlx::query_as::<_, TurnDetail>(
            r#"
            select timestamp, total_tokens, input_tokens, cached_input_tokens,
                   output_tokens, reasoning_output_tokens
            from token_events
            where session_id = ?1
            order by timestamp asc
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn session_path(&self, session_id: &str) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar::<_, String>(
            r#"
            select path
            from sessions
            where session_id = ?1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn dashboard_summary(&self) -> Result<DashboardSummary, sqlx::Error> {
        self.dashboard_summary_at(Utc::now()).await
    }

    pub async fn dashboard_summary_at(
        &self,
        now: DateTime<Utc>,
    ) -> Result<DashboardSummary, sqlx::Error> {
        let current_hour = hour_bucket_from_datetime(now);
        let today_start = day_start(now);
        let last_hour_start = (now - Duration::hours(1)).to_rfc3339_opts(SecondsFormat::Secs, true);
        let last_five_hours_start =
            (now - Duration::hours(5)).to_rfc3339_opts(SecondsFormat::Secs, true);
        let now_rfc3339 = now.to_rfc3339_opts(SecondsFormat::Secs, true);

        let today_total_tokens = sum_hourly_tokens(&self.pool, &today_start, &current_hour).await?;
        let last_hour_tokens = sum_event_tokens(&self.pool, &last_hour_start, &now_rfc3339).await?;
        let last_five_hours_tokens =
            sum_event_tokens(&self.pool, &last_five_hours_start, &now_rfc3339).await?;
        let active_session_count = sqlx::query_scalar::<_, i64>(
            r#"
            select count(distinct session_id)
            from token_events
            where timestamp >= ?1 and timestamp <= ?2
            "#,
        )
        .bind(&last_five_hours_start)
        .bind(&now_rfc3339)
        .fetch_one(&self.pool)
        .await?;
        let (input_tokens, output_tokens) = sqlx::query_as::<_, (i64, i64)>(
            r#"
            select coalesce(sum(input_tokens), 0), coalesce(sum(output_tokens), 0)
            from sessions
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DashboardSummary {
            today_total_tokens,
            last_hour_tokens,
            last_five_hours_tokens,
            active_session_count,
            input_tokens,
            output_tokens,
        })
    }

    pub async fn alerts(&self) -> Result<Vec<AlertItem>, sqlx::Error> {
        let mut alerts = Vec::new();

        let session_rows = sqlx::query_as::<_, (String, i64, String)>(
            r#"
            select session_id, total_tokens, last_seen_at
            from sessions
            where total_tokens > ?1
            "#,
        )
        .bind(SESSION_WARNING_TOKENS)
        .fetch_all(&self.pool)
        .await?;
        for (session_id, total_tokens, timestamp) in session_rows {
            if let Some(level) = token_level(
                total_tokens,
                SESSION_WARNING_TOKENS,
                SESSION_CRITICAL_TOKENS,
            ) {
                alerts.push(AlertItem {
                    level,
                    kind: AlertKind::HighSessionUsage,
                    message: format!(
                        "Session {session_id} used {total_tokens} tokens, exceeding the session threshold"
                    ),
                    timestamp,
                    session_id: Some(session_id),
                });
            }
        }

        let hourly_rows = sqlx::query_as::<_, (String, i64)>(
            r#"
            select bucket, total_tokens
            from hourly_buckets
            where total_tokens > ?1
            "#,
        )
        .bind(HOURLY_WARNING_TOKENS)
        .fetch_all(&self.pool)
        .await?;
        for (timestamp, total_tokens) in hourly_rows {
            if let Some(level) =
                token_level(total_tokens, HOURLY_WARNING_TOKENS, HOURLY_CRITICAL_TOKENS)
            {
                alerts.push(AlertItem {
                    level,
                    kind: AlertKind::HighHourlyUsage,
                    message: format!(
                        "Hour {timestamp} used {total_tokens} tokens, exceeding the hourly threshold"
                    ),
                    timestamp,
                    session_id: None,
                });
            }
        }

        let tool_rows = sqlx::query_as::<_, (String, String, i64)>(
            r#"
            select session_id, timestamp, output_bytes
            from tool_events
            where output_bytes > ?1
            "#,
        )
        .bind(TOOL_OUTPUT_WARNING_BYTES)
        .fetch_all(&self.pool)
        .await?;
        for (session_id, timestamp, output_bytes) in tool_rows {
            if let Some(level) = token_level(
                output_bytes,
                TOOL_OUTPUT_WARNING_BYTES,
                TOOL_OUTPUT_CRITICAL_BYTES,
            ) {
                alerts.push(AlertItem {
                    level,
                    kind: AlertKind::LargeToolOutput,
                    message: format!(
                        "Tool output in session {session_id} was {output_bytes} bytes, exceeding the tool output threshold"
                    ),
                    timestamp,
                    session_id: Some(session_id),
                });
            }
        }

        alerts.sort_by(|a, b| {
            b.timestamp
                .cmp(&a.timestamp)
                .then_with(|| b.level.cmp(&a.level))
                .then_with(|| a.message.cmp(&b.message))
        });
        Ok(alerts)
    }
}

fn parse_timestamp_utc(timestamp: &str) -> Result<DateTime<Utc>, sqlx::Error> {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| sqlx::Error::Protocol(format!("invalid token timestamp: {err}")))
}

fn hour_bucket_from_datetime(timestamp: DateTime<Utc>) -> String {
    timestamp
        .with_minute(0)
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .expect("zeroed hour timestamp should be valid")
        .to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn day_bucket_from_datetime(timestamp: DateTime<Utc>) -> String {
    timestamp.date_naive().format("%Y-%m-%d").to_string()
}

fn day_start(timestamp: DateTime<Utc>) -> String {
    DateTime::<Utc>::from_naive_utc_and_offset(
        timestamp
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .expect("midnight should be valid"),
        Utc,
    )
    .to_rfc3339_opts(SecondsFormat::Secs, true)
}

async fn sum_hourly_tokens(
    pool: &SqlitePool,
    start_bucket: &str,
    end_bucket: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        select coalesce(sum(total_tokens), 0)
        from hourly_buckets
        where bucket >= ?1 and bucket <= ?2
        "#,
    )
    .bind(start_bucket)
    .bind(end_bucket)
    .fetch_one(pool)
    .await
}

async fn sum_event_tokens(
    pool: &SqlitePool,
    start_timestamp: &str,
    end_timestamp: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        select coalesce(sum(total_tokens), 0)
        from token_events
        where timestamp >= ?1 and timestamp <= ?2
        "#,
    )
    .bind(start_timestamp)
    .bind(end_timestamp)
    .fetch_one(pool)
    .await
}
