pub mod alerts;
pub mod codex_log;
pub mod scanner;
pub mod usage_store;

use tauri::State;
use usage_store::{DashboardSummary, SessionSummary, TimeBucket, TurnDetail, UsageStore};

pub struct AppState {
    pub store: UsageStore,
}

mod commands {
    use super::*;

    #[tauri::command]
    pub async fn dashboard_summary(state: State<'_, AppState>) -> Result<DashboardSummary, String> {
        state
            .store
            .dashboard_summary()
            .await
            .map_err(|err| err.to_string())
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

    #[tauri::command]
    pub async fn daily_totals(state: State<'_, AppState>) -> Result<Vec<TimeBucket>, String> {
        state
            .store
            .daily_totals()
            .await
            .map_err(|err| err.to_string())
    }

    #[tauri::command]
    pub async fn session_turns(
        state: State<'_, AppState>,
        session_id: String,
    ) -> Result<Vec<TurnDetail>, String> {
        state
            .store
            .session_turns(&session_id)
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
            commands::hourly_totals,
            commands::daily_totals,
            commands::session_turns
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
