pub mod alerts;
pub mod codex_log;
pub mod scanner;
pub mod usage_store;
pub mod watcher;

use std::{error::Error, path::PathBuf};

use alerts::AlertItem;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, State, WindowEvent,
};
use usage_store::{DashboardSummary, SessionSummary, TimeBucket, TurnDetail, UsageStore};

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_OPEN_DASHBOARD_ID: &str = "open-dashboard";
const TRAY_QUIT_ID: &str = "quit";

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

    #[tauri::command]
    pub async fn list_alerts(state: State<'_, AppState>) -> Result<Vec<AlertItem>, String> {
        state.store.alerts().await.map_err(|err| err.to_string())
    }
}

fn show_dashboard(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(
        app,
        TRAY_OPEN_DASHBOARD_ID,
        "Open Dashboard",
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, TRAY_QUIT_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &separator, &quit])?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Codex Token Monitor")
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            TRAY_OPEN_DASHBOARD_ID => show_dashboard(app),
            TRAY_QUIT_ID => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    tray.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let store = tauri::async_runtime::block_on(async {
        let store = if let Some(path) = default_usage_store_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            UsageStore::open(&path.to_string_lossy()).await?
        } else {
            UsageStore::memory().await?
        };
        store.init().await?;
        Ok::<UsageStore, Box<dyn Error>>(store)
    })
    .expect("failed to initialize usage store");

    tauri::Builder::default()
        .manage(AppState { store })
        .setup(|app| {
            setup_tray(app)?;
            watcher::start_background_monitor(app.state::<AppState>().store.clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == MAIN_WINDOW_LABEL {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::dashboard_summary,
            commands::list_sessions,
            commands::hourly_totals,
            commands::daily_totals,
            commands::session_turns,
            commands::list_alerts
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

fn default_usage_store_path() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("codex-token-monitor").join("usage.sqlite3"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tray_menu_item_ids_are_stable() {
        assert_eq!(TRAY_OPEN_DASHBOARD_ID, "open-dashboard");
        assert_eq!(TRAY_QUIT_ID, "quit");
    }
}
