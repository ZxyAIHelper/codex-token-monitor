pub mod alerts;
pub mod codex_log;
pub mod scanner;
pub mod usage_store;

use serde::Serialize;
use tauri::State;
use usage_store::{SessionSummary, TimeBucket, UsageStore};

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

mod commands {
    use super::*;

    #[tauri::command]
    pub async fn dashboard_summary(
        _state: State<'_, AppState>,
    ) -> Result<DashboardSummary, String> {
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
        state.store.sessions().await.map_err(|err| err.to_string())
    }

    #[tauri::command]
    pub async fn hourly_totals(state: State<'_, AppState>) -> Result<Vec<TimeBucket>, String> {
        state
            .store
            .hourly_totals()
            .await
            .map_err(|err| err.to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = tauri::async_runtime::block_on(async {
        let store = UsageStore::memory().await?;
        store.init().await?;
        Ok::<UsageStore, sqlx::Error>(store)
    })
    .expect("failed to initialize usage store");

    tauri::Builder::default()
        .manage(AppState { store })
        .invoke_handler(tauri::generate_handler![
            commands::dashboard_summary,
            commands::list_sessions,
            commands::hourly_totals
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
