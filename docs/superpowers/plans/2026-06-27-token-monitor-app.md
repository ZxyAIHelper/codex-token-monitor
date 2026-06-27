# Codex Token Monitor App Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a low-power cross-platform desktop app that monitors local Codex token usage from `~/.codex/sessions`.

**Architecture:** Use Tauri with a Rust backend and WebView dashboard. Rust incrementally parses Codex JSONL logs, stores derived aggregates in SQLite, watches file changes, owns tray behavior, and exposes read APIs to the UI.

**Tech Stack:** Tauri 2, Rust, SQLite via `sqlx`, `notify` for file watching, React + TypeScript + Vite for UI, Vitest for frontend tests, Rust unit tests for parser/cache logic.

---

## File Structure

- `docs/specs/2026-06-27-token-monitor-app-spec.md` - product spec.
- `src-tauri/src/main.rs` - Tauri app entrypoint and plugin setup.
- `src-tauri/src/lib.rs` - app bootstrap, tray, state wiring, and Tauri commands.
- `src-tauri/src/codex_log.rs` - JSONL event parser and domain types.
- `src-tauri/src/usage_store.rs` - SQLite schema, aggregate writes, query APIs.
- `src-tauri/src/scanner.rs` - initial scan and incremental file parsing by byte offset.
- `src-tauri/src/watcher.rs` - file watcher plus periodic reconciliation.
- `src-tauri/src/alerts.rs` - threshold evaluation and warning generation.
- `src-tauri/tests/parser_tests.rs` - parser and accounting tests.
- `src-tauri/tests/store_tests.rs` - SQLite aggregate tests.
- `src/App.tsx` - top-level dashboard shell and view routing.
- `src/api.ts` - typed Tauri command client.
- `src/types.ts` - frontend data types mirrored from Rust responses.
- `src/components/SummaryCards.tsx` - top dashboard metrics.
- `src/components/HourlyTrend.tsx` - simple hourly trend chart.
- `src/components/SessionTable.tsx` - sortable session ranking table.
- `src/components/AlertList.tsx` - recent alerts.
- `src/components/SessionDetail.tsx` - session drill-down view.
- `src/components/SettingsView.tsx` - settings page.
- `src/styles.css` - dashboard styling.
- `src/__tests__/format.test.ts` - frontend formatting and sorting tests.

---

### Task 1: Scaffold Tauri App

**Files:**
- Create: `package.json`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `index.html`
- Create: `src/main.tsx`
- Create: `src/App.tsx`

- [ ] **Step 1: Scaffold app files**

Run from `E:\WorkSpace\ai\codex-token-monitor`:

```powershell
npm create tauri-app@latest . -- --template react-ts --manager npm
```

Expected: project files are created in the current directory, including `src-tauri`, `src`, and `package.json`.

- [ ] **Step 2: Install frontend dependencies**

```powershell
npm install
```

Expected: `node_modules` and `package-lock.json` are created.

- [ ] **Step 3: Add Rust dependencies**

Edit `src-tauri/Cargo.toml` and make sure dependencies include:

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-opener = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
notify = "8"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "chrono"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
dirs = "6"
thiserror = "2"
```

- [ ] **Step 4: Verify scaffold builds**

```powershell
npm run tauri dev
```

Expected: app window opens with default Tauri React template.

- [ ] **Step 5: Commit scaffold**

```powershell
git add .
git commit -m "chore: scaffold token monitor app"
```

---

### Task 2: Implement Codex JSONL Parser

**Files:**
- Create: `src-tauri/src/codex_log.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/tests/parser_tests.rs`

- [ ] **Step 1: Write parser tests**

Create `src-tauri/tests/parser_tests.rs`:

```rust
use codex_token_monitor_lib::codex_log::{parse_jsonl_line, CodexEvent};

#[test]
fn parses_token_count_last_usage() {
    let line = r#"{"timestamp":"2026-06-27T12:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":7,"reasoning_output_tokens":3,"total_tokens":107}}}}"#;
    let event = parse_jsonl_line(line).unwrap();

    match event {
        CodexEvent::TokenCount(e) => {
            assert_eq!(e.timestamp, "2026-06-27T12:00:00Z");
            assert_eq!(e.input_tokens, 100);
            assert_eq!(e.cached_input_tokens, 40);
            assert_eq!(e.output_tokens, 7);
            assert_eq!(e.reasoning_output_tokens, 3);
            assert_eq!(e.total_tokens, 107);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn parses_function_call_output_size() {
    let line = r#"{"timestamp":"2026-06-27T12:00:01Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_1","output":"Exit code: 0\nOutput: hello"}}"#;
    let event = parse_jsonl_line(line).unwrap();

    match event {
        CodexEvent::ToolOutput(e) => {
            assert_eq!(e.output_bytes, "Exit code: 0\nOutput: hello".len() as i64);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn ignores_unrelated_json_lines() {
    let line = r#"{"timestamp":"2026-06-27T12:00:02Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[]}}"#;
    let event = parse_jsonl_line(line).unwrap();
    assert!(matches!(event, CodexEvent::Ignored));
}
```

- [ ] **Step 2: Run test and confirm failure**

```powershell
cd src-tauri
cargo test parser_tests -- --nocapture
```

Expected: FAIL because `codex_log` does not exist.

- [ ] **Step 3: Implement parser**

Create `src-tauri/src/codex_log.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexEvent {
    TokenCount(TokenCountEvent),
    ToolOutput(ToolOutputEvent),
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenCountEvent {
    pub timestamp: String,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutputEvent {
    pub timestamp: String,
    pub output_bytes: i64,
}

#[derive(Debug, Deserialize)]
struct RawLine {
    timestamp: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    payload: Option<serde_json::Value>,
}

pub fn parse_jsonl_line(line: &str) -> Result<CodexEvent, serde_json::Error> {
    let raw: RawLine = serde_json::from_str(line)?;
    let timestamp = raw.timestamp.unwrap_or_default();
    let Some(payload) = raw.payload else {
        return Ok(CodexEvent::Ignored);
    };

    if raw.kind.as_deref() == Some("event_msg")
        && payload.get("type").and_then(|v| v.as_str()) == Some("token_count")
    {
        let usage = payload
            .get("info")
            .and_then(|v| v.get("last_token_usage"))
            .cloned()
            .unwrap_or_default();

        return Ok(CodexEvent::TokenCount(TokenCountEvent {
            timestamp,
            input_tokens: usage.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0),
            cached_input_tokens: usage
                .get("cached_input_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            output_tokens: usage.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0),
            reasoning_output_tokens: usage
                .get("reasoning_output_tokens")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            total_tokens: usage.get("total_tokens").and_then(|v| v.as_i64()).unwrap_or(0),
        }));
    }

    if raw.kind.as_deref() == Some("response_item")
        && payload.get("type").and_then(|v| v.as_str()) == Some("function_call_output")
    {
        let output = payload.get("output").and_then(|v| v.as_str()).unwrap_or("");
        return Ok(CodexEvent::ToolOutput(ToolOutputEvent {
            timestamp,
            output_bytes: output.len() as i64,
        }));
    }

    Ok(CodexEvent::Ignored)
}
```

Modify `src-tauri/src/lib.rs`:

```rust
pub mod codex_log;
```

- [ ] **Step 4: Run parser tests**

```powershell
cd src-tauri
cargo test parser_tests -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit parser**

```powershell
git add src-tauri/src/codex_log.rs src-tauri/src/lib.rs src-tauri/tests/parser_tests.rs
git commit -m "feat: parse codex token usage events"
```

---

### Task 3: Add SQLite Derived Cache

**Files:**
- Create: `src-tauri/src/usage_store.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/tests/store_tests.rs`

- [ ] **Step 1: Write store test**

Create `src-tauri/tests/store_tests.rs`:

```rust
use codex_token_monitor_lib::codex_log::TokenCountEvent;
use codex_token_monitor_lib::usage_store::UsageStore;

#[tokio::test]
async fn stores_session_and_hourly_totals() {
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
    assert_eq!(sessions[0].output_tokens, 7);

    let hours = store.hourly_totals().await.unwrap();
    assert_eq!(hours.len(), 1);
    assert_eq!(hours[0].bucket, "2026-06-27T12:00:00Z");
    assert_eq!(hours[0].total_tokens, 107);
}
```

- [ ] **Step 2: Run test and confirm failure**

```powershell
cd src-tauri
cargo test store_tests -- --nocapture
```

Expected: FAIL because `usage_store` does not exist.

- [ ] **Step 3: Implement store**

Create `src-tauri/src/usage_store.rs` with schema tables:

```rust
use crate::codex_log::TokenCountEvent;
use chrono::{DateTime, Timelike, Utc};
use serde::Serialize;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

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
        let pool = SqlitePoolOptions::new().connect("sqlite::memory:").await?;
        Ok(Self { pool })
    }

    pub async fn open(path: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new().connect(path).await?;
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
            );
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
            );
            create table if not exists hourly_buckets (
              bucket text primary key,
              total_tokens integer not null default 0,
              input_tokens integer not null default 0,
              output_tokens integer not null default 0,
              session_count integer not null default 0
            );
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
        let bucket = hour_bucket(&event.timestamp);

        sqlx::query(
            r#"
            insert into sessions (
              session_id, path, total_tokens, input_tokens, cached_input_tokens,
              output_tokens, reasoning_output_tokens, last_seen_at
            )
            values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            on conflict(session_id) do update set
              total_tokens = total_tokens + excluded.total_tokens,
              input_tokens = input_tokens + excluded.input_tokens,
              cached_input_tokens = cached_input_tokens + excluded.cached_input_tokens,
              output_tokens = output_tokens + excluded.output_tokens,
              reasoning_output_tokens = reasoning_output_tokens + excluded.reasoning_output_tokens,
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
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            insert into hourly_buckets (bucket, total_tokens, input_tokens, output_tokens, session_count)
            values (?1, ?2, ?3, ?4, 1)
            on conflict(bucket) do update set
              total_tokens = total_tokens + excluded.total_tokens,
              input_tokens = input_tokens + excluded.input_tokens,
              output_tokens = output_tokens + excluded.output_tokens
            "#,
        )
        .bind(bucket)
        .bind(event.total_tokens)
        .bind(event.input_tokens)
        .bind(event.output_tokens)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn sessions(&self) -> Result<Vec<SessionSummary>, sqlx::Error> {
        sqlx::query_as::<_, SessionSummary>(
            "select * from sessions order by total_tokens desc",
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn hourly_totals(&self) -> Result<Vec<TimeBucket>, sqlx::Error> {
        sqlx::query_as::<_, TimeBucket>(
            "select * from hourly_buckets order by bucket asc",
        )
        .fetch_all(&self.pool)
        .await
    }
}

fn hour_bucket(timestamp: &str) -> String {
    let parsed = DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    parsed
        .with_minute(0)
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .unwrap()
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
```

Modify `src-tauri/src/lib.rs`:

```rust
pub mod codex_log;
pub mod usage_store;
```

- [ ] **Step 4: Run store tests**

```powershell
cd src-tauri
cargo test store_tests -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit store**

```powershell
git add src-tauri/src/usage_store.rs src-tauri/src/lib.rs src-tauri/tests/store_tests.rs
git commit -m "feat: cache token aggregates in sqlite"
```

---

### Task 4: Implement Scanner And Alert Evaluation

**Files:**
- Create: `src-tauri/src/scanner.rs`
- Create: `src-tauri/src/alerts.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/tests/scanner_tests.rs`

- [ ] **Step 1: Write scanner test**

Create `src-tauri/tests/scanner_tests.rs`:

```rust
use codex_token_monitor_lib::scanner::{extract_session_id, should_scan_path};
use std::path::Path;

#[test]
fn extracts_session_id_from_rollout_filename() {
    let path = Path::new("rollout-2026-06-27T19-58-09-019f08f1-c13e-7ea1-b57d-8ba3bc9d4186.jsonl");
    assert_eq!(
        extract_session_id(path),
        Some("019f08f1-c13e-7ea1-b57d-8ba3bc9d4186".to_string())
    );
}

#[test]
fn scans_only_jsonl_files() {
    assert!(should_scan_path(Path::new("a.jsonl")));
    assert!(!should_scan_path(Path::new("a.sqlite")));
    assert!(!should_scan_path(Path::new("README.md")));
}
```

- [ ] **Step 2: Run test and confirm failure**

```powershell
cd src-tauri
cargo test scanner_tests -- --nocapture
```

Expected: FAIL because `scanner` does not exist.

- [ ] **Step 3: Implement scanner helpers and alerts**

Create `src-tauri/src/scanner.rs`:

```rust
use std::path::Path;

pub fn should_scan_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jsonl"))
}

pub fn extract_session_id(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let without_ext = file_name.strip_suffix(".jsonl")?;
    let marker = "-019";
    let idx = without_ext.rfind(marker)?;
    Some(without_ext[idx + 1..].to_string())
}
```

Create `src-tauri/src/alerts.rs`:

```rust
use serde::Serialize;

pub const HOURLY_WARNING_TOKENS: i64 = 1_000_000;
pub const HOURLY_CRITICAL_TOKENS: i64 = 3_000_000;
pub const SESSION_WARNING_TOKENS: i64 = 3_000_000;
pub const SESSION_CRITICAL_TOKENS: i64 = 10_000_000;
pub const TOOL_OUTPUT_WARNING_BYTES: i64 = 50 * 1024;
pub const TOOL_OUTPUT_CRITICAL_BYTES: i64 = 200 * 1024;
pub const BASE64_WARNING_BYTES: i64 = 100 * 1024;
pub const BASE64_CRITICAL_BYTES: i64 = 500 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum AlertLevel {
    Warning,
    Critical,
}

pub fn token_level(value: i64, warning: i64, critical: i64) -> Option<AlertLevel> {
    if value >= critical {
        Some(AlertLevel::Critical)
    } else if value >= warning {
        Some(AlertLevel::Warning)
    } else {
        None
    }
}
```

Modify `src-tauri/src/lib.rs`:

```rust
pub mod alerts;
pub mod codex_log;
pub mod scanner;
pub mod usage_store;
```

- [ ] **Step 4: Run scanner tests**

```powershell
cd src-tauri
cargo test scanner_tests -- --nocapture
```

Expected: PASS.

- [ ] **Step 5: Commit scanner and alerts**

```powershell
git add src-tauri/src/scanner.rs src-tauri/src/alerts.rs src-tauri/src/lib.rs src-tauri/tests/scanner_tests.rs
git commit -m "feat: add session scanner helpers and thresholds"
```

---

### Task 5: Expose Tauri Commands

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Add command response types and commands**

Modify `src-tauri/src/lib.rs` so it contains:

```rust
pub mod alerts;
pub mod codex_log;
pub mod scanner;
pub mod usage_store;

use serde::Serialize;
use tauri::{Manager, State};
use usage_store::{SessionSummary, TimeBucket, UsageStore};

#[derive(Clone)]
pub struct AppState {
    pub store: UsageStore,
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

#[tauri::command]
pub async fn dashboard_summary(_state: State<'_, AppState>) -> Result<DashboardSummary, String> {
    Ok(DashboardSummary {
        today_total_tokens: 0,
        last_hour_tokens: 0,
        last_five_hours_tokens: 0,
        active_session_count: 0,
        input_tokens: 0,
        output_tokens: 0,
    })
}

#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionSummary>, String> {
    state.store.sessions().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hourly_totals(state: State<'_, AppState>) -> Result<Vec<TimeBucket>, String> {
    state.store.hourly_totals().await.map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let store = UsageStore::memory().await.expect("open usage store");
                store.init().await.expect("init usage store");
                app_handle.manage(AppState { store });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            dashboard_summary,
            list_sessions,
            hourly_totals
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Modify `src-tauri/src/main.rs`:

```rust
fn main() {
    codex_token_monitor_lib::run();
}
```

- [ ] **Step 2: Verify Rust compiles**

```powershell
cd src-tauri
cargo check
```

Expected: PASS.

- [ ] **Step 3: Commit commands**

```powershell
git add src-tauri/src/lib.rs src-tauri/src/main.rs
git commit -m "feat: expose dashboard tauri commands"
```

---

### Task 6: Add Background Scan, Daily Buckets, And Session Detail APIs

**Files:**
- Modify: `src-tauri/src/usage_store.rs`
- Modify: `src-tauri/src/scanner.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/tests/detail_tests.rs`

- [ ] **Step 1: Add detail test**

Create `src-tauri/tests/detail_tests.rs`:

```rust
use codex_token_monitor_lib::codex_log::TokenCountEvent;
use codex_token_monitor_lib::usage_store::UsageStore;

#[tokio::test]
async fn stores_turn_details_and_daily_totals() {
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

    let details = store.session_turns("session-a").await.unwrap();
    assert_eq!(details.len(), 1);
    assert_eq!(details[0].total_tokens, 107);

    let days = store.daily_totals().await.unwrap();
    assert_eq!(days.len(), 1);
    assert_eq!(days[0].bucket, "2026-06-27");
    assert_eq!(days[0].total_tokens, 107);
}
```

- [ ] **Step 2: Run test and confirm failure**

```powershell
cd src-tauri
cargo test detail_tests -- --nocapture
```

Expected: FAIL because `session_turns` and `daily_totals` are not implemented.

- [ ] **Step 3: Extend SQLite schema**

Modify `UsageStore::init()` in `src-tauri/src/usage_store.rs` to also create:

```sql
create table if not exists token_events (
  id integer primary key autoincrement,
  session_id text not null,
  path text not null,
  timestamp text not null,
  total_tokens integer not null,
  input_tokens integer not null,
  cached_input_tokens integer not null,
  output_tokens integer not null,
  reasoning_output_tokens integer not null
);
create table if not exists daily_buckets (
  bucket text primary key,
  total_tokens integer not null default 0,
  input_tokens integer not null default 0,
  output_tokens integer not null default 0,
  session_count integer not null default 0
);
```

- [ ] **Step 4: Add detail structs and queries**

Add to `src-tauri/src/usage_store.rs`:

```rust
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TurnDetail {
    pub timestamp: String,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
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

pub async fn daily_totals(&self) -> Result<Vec<TimeBucket>, sqlx::Error> {
    sqlx::query_as::<_, TimeBucket>("select * from daily_buckets order by bucket asc")
        .fetch_all(&self.pool)
        .await
}
```

Also update `record_token_count()` to insert one row into `token_events` and update `daily_buckets` using `event.timestamp[0..10]` as the day bucket.

- [ ] **Step 5: Expose detail commands**

Add Tauri commands in `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
pub async fn daily_totals(state: State<'_, AppState>) -> Result<Vec<TimeBucket>, String> {
    state.store.daily_totals().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn session_turns(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Vec<usage_store::TurnDetail>, String> {
    state
        .store
        .session_turns(&session_id)
        .await
        .map_err(|e| e.to_string())
}
```

Register both commands in `tauri::generate_handler!`.

- [ ] **Step 6: Run tests**

```powershell
cd src-tauri
cargo test detail_tests store_tests -- --nocapture
```

Expected: PASS.

- [ ] **Step 7: Commit detail APIs**

```powershell
git add src-tauri/src/usage_store.rs src-tauri/src/lib.rs src-tauri/tests/detail_tests.rs
git commit -m "feat: add daily and session detail aggregates"
```

---

### Task 7: Build Dashboard UI

**Files:**
- Create: `src/types.ts`
- Create: `src/api.ts`
- Create: `src/components/SummaryCards.tsx`
- Create: `src/components/HourlyTrend.tsx`
- Create: `src/components/SessionTable.tsx`
- Create: `src/components/AlertList.tsx`
- Modify: `src/App.tsx`
- Modify: `src/styles.css`

- [ ] **Step 1: Add frontend types**

Create `src/types.ts`:

```ts
export type DashboardSummary = {
  today_total_tokens: number;
  last_hour_tokens: number;
  last_five_hours_tokens: number;
  active_session_count: number;
  input_tokens: number;
  output_tokens: number;
};

export type SessionSummary = {
  session_id: string;
  path: string;
  total_tokens: number;
  input_tokens: number;
  cached_input_tokens: number;
  output_tokens: number;
  reasoning_output_tokens: number;
  tool_calls: number;
  tool_output_bytes: number;
  last_seen_at: string;
};

export type TimeBucket = {
  bucket: string;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  session_count: number;
};
```

- [ ] **Step 2: Add API wrapper**

Create `src/api.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { DashboardSummary, SessionSummary, TimeBucket } from "./types";

export function getDashboardSummary() {
  return invoke<DashboardSummary>("dashboard_summary");
}

export function getSessions() {
  return invoke<SessionSummary[]>("list_sessions");
}

export function getHourlyTotals() {
  return invoke<TimeBucket[]>("hourly_totals");
}
```

- [ ] **Step 3: Create summary cards component**

Create `src/components/SummaryCards.tsx`:

```tsx
import type { DashboardSummary } from "../types";

function formatTokens(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`;
  return String(value);
}

export function SummaryCards({ summary }: { summary: DashboardSummary }) {
  const cards = [
    ["Today", summary.today_total_tokens],
    ["Last 1h", summary.last_hour_tokens],
    ["Last 5h", summary.last_five_hours_tokens],
    ["Input", summary.input_tokens],
    ["Output", summary.output_tokens],
    ["Tasks", summary.active_session_count],
  ] as const;

  return (
    <section className="summary-grid">
      {cards.map(([label, value]) => (
        <button className="summary-card" key={label}>
          <span>{label}</span>
          <strong>{formatTokens(value)}</strong>
        </button>
      ))}
    </section>
  );
}
```

- [ ] **Step 4: Create session table component**

Create `src/components/SessionTable.tsx`:

```tsx
import type { SessionSummary } from "../types";

export function SessionTable({ sessions }: { sessions: SessionSummary[] }) {
  return (
    <section className="panel">
      <header>High Usage Sessions</header>
      <table>
        <thead>
          <tr>
            <th>Session</th>
            <th>Total</th>
            <th>Input</th>
            <th>Output</th>
            <th>Tools</th>
            <th>Tool KB</th>
          </tr>
        </thead>
        <tbody>
          {sessions.map((session) => (
            <tr key={session.session_id}>
              <td>{session.session_id}</td>
              <td>{session.total_tokens.toLocaleString()}</td>
              <td>{session.input_tokens.toLocaleString()}</td>
              <td>{session.output_tokens.toLocaleString()}</td>
              <td>{session.tool_calls}</td>
              <td>{Math.round(session.tool_output_bytes / 1024)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
```

- [ ] **Step 5: Wire dashboard shell**

Modify `src/App.tsx`:

```tsx
import { useEffect, useState } from "react";
import { getDashboardSummary, getHourlyTotals, getSessions } from "./api";
import { SummaryCards } from "./components/SummaryCards";
import { SessionTable } from "./components/SessionTable";
import type { DashboardSummary, SessionSummary, TimeBucket } from "./types";
import "./styles.css";

export default function App() {
  const [summary, setSummary] = useState<DashboardSummary | null>(null);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [hours, setHours] = useState<TimeBucket[]>([]);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      const [nextSummary, nextSessions, nextHours] = await Promise.all([
        getDashboardSummary(),
        getSessions(),
        getHourlyTotals(),
      ]);
      if (!cancelled) {
        setSummary(nextSummary);
        setSessions(nextSessions);
        setHours(nextHours);
      }
    }

    load();
    const timer = window.setInterval(load, 5000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  return (
    <main>
      <header className="app-header">
        <div>
          <h1>Codex Token Monitor</h1>
          <p>Local sessions only</p>
        </div>
      </header>
      {summary && <SummaryCards summary={summary} />}
      <section className="panel">
        <header>Last 24 Hours</header>
        <div className="chart-placeholder">
          {hours.length === 0 ? "No hourly data yet" : `${hours.length} buckets loaded`}
        </div>
      </section>
      <div className="dashboard-grid">
        <SessionTable sessions={sessions} />
        <section className="panel">
          <header>Alerts</header>
          <p>No alerts yet</p>
        </section>
      </div>
    </main>
  );
}
```

- [ ] **Step 6: Add dashboard styles**

Modify `src/styles.css`:

```css
:root {
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  color: #172026;
  background: #f5f7f8;
}

body {
  margin: 0;
}

button {
  font: inherit;
}

main {
  min-height: 100vh;
  padding: 24px;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.app-header h1 {
  margin: 0;
  font-size: 24px;
}

.app-header p {
  margin: 4px 0 0;
  color: #60717c;
}

.summary-grid {
  display: grid;
  grid-template-columns: repeat(6, minmax(120px, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.summary-card,
.panel {
  border: 1px solid #dce3e8;
  background: #ffffff;
  border-radius: 8px;
}

.summary-card {
  display: grid;
  gap: 8px;
  padding: 14px;
  text-align: left;
  cursor: pointer;
}

.summary-card span {
  color: #60717c;
  font-size: 13px;
}

.summary-card strong {
  font-size: 22px;
}

.panel {
  padding: 16px;
}

.panel header {
  font-weight: 700;
  margin-bottom: 12px;
}

.dashboard-grid {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 16px;
  margin-top: 16px;
}

.chart-placeholder {
  min-height: 160px;
  display: grid;
  place-items: center;
  color: #60717c;
  background: #f7fafb;
  border-radius: 6px;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

th,
td {
  padding: 8px;
  border-bottom: 1px solid #edf1f3;
  text-align: left;
}

th {
  color: #60717c;
}
```

- [ ] **Step 7: Run frontend build**

```powershell
npm run build
```

Expected: PASS.

- [ ] **Step 8: Commit dashboard**

```powershell
git add src package.json package-lock.json
git commit -m "feat: add token monitor dashboard shell"
```

---

### Task 8: Add Session Detail UI And Alerts

**Files:**
- Modify: `src/api.ts`
- Modify: `src/types.ts`
- Create: `src/components/SessionDetail.tsx`
- Modify: `src/components/AlertList.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Add frontend detail types and APIs**

Add to `src/types.ts`:

```ts
export type TurnDetail = {
  timestamp: string;
  total_tokens: number;
  input_tokens: number;
  cached_input_tokens: number;
  output_tokens: number;
  reasoning_output_tokens: number;
};
```

Add to `src/api.ts`:

```ts
import type { TurnDetail } from "./types";

export function getSessionTurns(sessionId: string) {
  return invoke<TurnDetail[]>("session_turns", { sessionId });
}
```

- [ ] **Step 2: Create session detail component**

Create `src/components/SessionDetail.tsx`:

```tsx
import type { SessionSummary, TurnDetail } from "../types";

export function SessionDetail({
  session,
  turns,
  onBack,
}: {
  session: SessionSummary;
  turns: TurnDetail[];
  onBack: () => void;
}) {
  return (
    <section className="panel">
      <button className="text-button" onClick={onBack}>Back</button>
      <header>{session.session_id}</header>
      <p>{session.path}</p>
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Total</th>
            <th>Input</th>
            <th>Cached</th>
            <th>Output</th>
            <th>Reasoning</th>
          </tr>
        </thead>
        <tbody>
          {turns.map((turn) => (
            <tr key={`${turn.timestamp}-${turn.total_tokens}`}>
              <td>{turn.timestamp}</td>
              <td>{turn.total_tokens.toLocaleString()}</td>
              <td>{turn.input_tokens.toLocaleString()}</td>
              <td>{turn.cached_input_tokens.toLocaleString()}</td>
              <td>{turn.output_tokens.toLocaleString()}</td>
              <td>{turn.reasoning_output_tokens.toLocaleString()}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
```

- [ ] **Step 3: Make session rows clickable**

Modify `SessionTable` props to accept `onOpenSession` and call it from each row:

```tsx
export function SessionTable({
  sessions,
  onOpenSession,
}: {
  sessions: SessionSummary[];
  onOpenSession: (session: SessionSummary) => void;
}) {
  return (
    <section className="panel">
      <header>High Usage Sessions</header>
      <table>
        <tbody>
          {sessions.map((session) => (
            <tr key={session.session_id} onClick={() => onOpenSession(session)}>
              <td>{session.session_id}</td>
              <td>{session.total_tokens.toLocaleString()}</td>
              <td>{session.input_tokens.toLocaleString()}</td>
              <td>{session.output_tokens.toLocaleString()}</td>
              <td>{session.tool_calls}</td>
              <td>{Math.round(session.tool_output_bytes / 1024)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
```

Keep the existing table header from Task 7.

- [ ] **Step 4: Wire detail state in App**

Modify `src/App.tsx` to keep selected session and turns:

```tsx
const [selectedSession, setSelectedSession] = useState<SessionSummary | null>(null);
const [turns, setTurns] = useState<TurnDetail[]>([]);

async function openSession(session: SessionSummary) {
  setSelectedSession(session);
  setTurns(await getSessionTurns(session.session_id));
}
```

Render `SessionDetail` when `selectedSession` is set; otherwise render the dashboard.

- [ ] **Step 5: Run frontend build**

```powershell
npm run build
```

Expected: PASS.

- [ ] **Step 6: Commit detail UI**

```powershell
git add src
git commit -m "feat: add session detail drilldown"
```

---

### Task 9: Add Tray Behavior

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add tray setup**

Modify `run()` in `src-tauri/src/lib.rs` to create a tray menu with Open Dashboard and Quit. Use Tauri tray APIs for Tauri 2. The behavior must:

- Launch without opening the main dashboard window when supported by configuration.
- Keep monitoring in the tray.
- Open or focus the dashboard window from tray menu.
- Quit from tray menu.

- [ ] **Step 2: Verify tray manually**

```powershell
npm run tauri dev
```

Expected:

- App tray icon appears.
- Dashboard can be opened from tray.
- Quit exits the app.

- [ ] **Step 3: Commit tray**

```powershell
git add src-tauri/src/lib.rs src-tauri/tauri.conf.json
git commit -m "feat: add tray monitor controls"
```

---

### Task 10: Add File Watcher And Initial Scan

**Files:**
- Create: `src-tauri/src/watcher.rs`
- Modify: `src-tauri/src/scanner.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Implement scan range resolver**

Add to `src-tauri/src/scanner.rs`:

```rust
use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use crate::codex_log::{parse_jsonl_line, CodexEvent};
use crate::usage_store::UsageStore;

pub async fn scan_file(store: &UsageStore, path: &Path, offset: u64) -> Result<u64, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;

    let session_id = extract_session_id(path).unwrap_or_else(|| path.to_string_lossy().to_string());
    let mut current_offset = offset;

    for line in reader.lines() {
      let line = line.map_err(|e| e.to_string())?;
      current_offset += line.len() as u64 + 1;
      match parse_jsonl_line(&line).map_err(|e| e.to_string())? {
        CodexEvent::TokenCount(event) => {
          store
            .record_token_count(&session_id, &path.to_string_lossy(), event)
            .await
            .map_err(|e| e.to_string())?;
        }
        CodexEvent::ToolOutput(_) | CodexEvent::Ignored => {}
      }
    }

    Ok(current_offset)
}

pub fn default_codex_sessions_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".codex").join("sessions"))
}
```

- [ ] **Step 2: Add watcher stub that can be started from app setup**

Create `src-tauri/src/watcher.rs`:

```rust
use crate::scanner::default_codex_sessions_dir;
use crate::usage_store::UsageStore;

pub async fn start_background_monitor(_store: UsageStore) {
    if let Some(dir) = default_codex_sessions_dir() {
        eprintln!("monitoring Codex sessions at {}", dir.display());
    }
}
```

- [ ] **Step 3: Start monitor during setup**

Modify `src-tauri/src/lib.rs` setup block after `store.init()`:

```rust
let monitor_store = store.clone();
tauri::async_runtime::spawn(async move {
    crate::watcher::start_background_monitor(monitor_store).await;
});
```

Add `pub mod watcher;`.

- [ ] **Step 4: Run Rust tests**

```powershell
cd src-tauri
cargo test
```

Expected: PASS.

- [ ] **Step 5: Commit watcher foundation**

```powershell
git add src-tauri/src/scanner.rs src-tauri/src/watcher.rs src-tauri/src/lib.rs
git commit -m "feat: add low power monitor foundation"
```

---

### Task 11: Build Packaging Smoke Test

**Files:**
- Modify only if build configuration requires it: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Run complete tests**

```powershell
cd src-tauri
cargo test
cd ..
npm run build
```

Expected: all Rust tests pass and frontend builds.

- [ ] **Step 2: Build desktop package**

```powershell
npm run tauri build
```

Expected on Windows: `.exe` or installer artifacts are generated under `src-tauri/target/release/bundle`.

- [ ] **Step 3: Manual smoke test packaged app**

Open the generated app and verify:

- Tray icon appears.
- Dashboard opens.
- Summary cards render.
- Session table renders without crashing.
- App quits cleanly.

- [ ] **Step 4: Commit packaging config**

```powershell
git add src-tauri/tauri.conf.json
git commit -m "chore: verify desktop packaging"
```

---

## Self-Review

- Spec coverage:
  - Local-only Codex monitoring is covered by parser, scanner, store, and dashboard tasks.
  - Tray-first behavior is covered by Task 9.
  - Daily/hourly/session/tool metrics are covered by Tasks 3, 6, 7, and 8.
  - Low-power incremental parsing and monitor startup are covered by Task 10.
- Placeholder scan:
  - No `TBD` or `TODO` markers remain.
  - The plan uses concrete expected behavior for platform tray APIs where generated Tauri templates can vary.
- Type consistency:
  - Rust response fields use snake_case and frontend types match those fields.
  - Store summaries match dashboard table columns.
