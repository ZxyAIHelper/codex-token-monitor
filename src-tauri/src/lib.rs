pub mod alerts;
pub mod codex_log;
pub mod scanner;
pub mod session_detail;
pub mod usage_store;
pub mod watcher;

use std::{
    error::Error,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use alerts::AlertItem;
use codex_log::{read_message_details, MessageDetail};
use session_detail::{group_model_requests, ModelRequestDetail};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager, State, WindowEvent, Wry,
};
use usage_store::{DashboardSummary, SessionSummary, TimeBucket, TurnDetail, UsageStore};

const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_ID: &str = "codex-token-monitor";
const TRAY_OPEN_DASHBOARD_ID: &str = "open-dashboard";
const TRAY_TODAY_SUMMARY_ID: &str = "today-summary";
const TRAY_TOGGLE_MONITORING_ID: &str = "toggle-monitoring";
const TRAY_QUIT_ID: &str = "quit";

pub struct AppState {
    pub store: UsageStore,
    pub monitor_paused: Arc<AtomicBool>,
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
    pub async fn session_messages(
        state: State<'_, AppState>,
        session_id: String,
    ) -> Result<Vec<MessageDetail>, String> {
        let Some(path) = state
            .store
            .session_path(&session_id)
            .await
            .map_err(|err| err.to_string())?
        else {
            return Ok(Vec::new());
        };

        read_message_details(PathBuf::from(path).as_path())
    }

    #[tauri::command]
    pub async fn session_model_requests(
        state: State<'_, AppState>,
        session_id: String,
    ) -> Result<Vec<ModelRequestDetail>, String> {
        let turns = state
            .store
            .session_turns(&session_id)
            .await
            .map_err(|err| err.to_string())?;
        let Some(path) = state
            .store
            .session_path(&session_id)
            .await
            .map_err(|err| err.to_string())?
        else {
            return Ok(group_model_requests(turns, Vec::new()));
        };
        let messages = read_message_details(PathBuf::from(path).as_path())?;

        Ok(group_model_requests(turns, messages))
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

fn new_monitor_paused_flag() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

fn is_monitoring_paused(paused: &AtomicBool) -> bool {
    paused.load(Ordering::Relaxed)
}

fn set_monitoring_paused(paused: &AtomicBool, value: bool) -> bool {
    paused.store(value, Ordering::Relaxed);
    value
}

fn format_compact_count(value: i64) -> String {
    let abs = value.abs();
    if abs >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if abs >= 1_000 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

fn format_tray_summary(summary: &DashboardSummary) -> String {
    format!(
        "Today: {} tokens, {} tasks",
        format_compact_count(summary.today_total_tokens),
        summary.active_session_count
    )
}

fn refresh_tray_summary(app: &tauri::AppHandle, summary_item: MenuItem<Wry>) {
    let app = app.clone();
    let store = app.state::<AppState>().store.clone();

    tauri::async_runtime::spawn(async move {
        match store.dashboard_summary().await {
            Ok(summary) => {
                let text = format_tray_summary(&summary);
                let _ = summary_item.set_text(format!("Today's Summary: {text}"));
                if let Some(tray) = app.tray_by_id(TRAY_ID) {
                    let _ = tray.set_tooltip(Some(format!("Codex Token Monitor - {text}")));
                }
            }
            Err(err) => {
                let _ = summary_item.set_text("Today's Summary: unavailable");
                if let Some(tray) = app.tray_by_id(TRAY_ID) {
                    let _ = tray.set_tooltip(Some(format!(
                        "Codex Token Monitor - summary unavailable: {err}"
                    )));
                }
            }
        }
    });
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(
        app,
        TRAY_OPEN_DASHBOARD_ID,
        "Open Dashboard",
        true,
        None::<&str>,
    )?;
    let today_summary = MenuItem::with_id(
        app,
        TRAY_TODAY_SUMMARY_ID,
        "Today's Summary: loading...",
        true,
        None::<&str>,
    )?;
    let toggle_monitoring = MenuItem::with_id(
        app,
        TRAY_TOGGLE_MONITORING_ID,
        "Pause Monitoring",
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, TRAY_QUIT_ID, "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&open, &today_summary, &toggle_monitoring, &separator, &quit],
    )?;

    let summary_for_handler = today_summary.clone();
    let toggle_for_handler = toggle_monitoring.clone();

    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("Codex Token Monitor")
        .show_menu_on_left_click(true)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            TRAY_OPEN_DASHBOARD_ID => {
                refresh_tray_summary(app, summary_for_handler.clone());
                show_dashboard(app);
            }
            TRAY_TODAY_SUMMARY_ID => {
                refresh_tray_summary(app, summary_for_handler.clone());
                show_dashboard(app);
            }
            TRAY_TOGGLE_MONITORING_ID => {
                let state = app.state::<AppState>();
                let next_paused = !is_monitoring_paused(&state.monitor_paused);
                set_monitoring_paused(&state.monitor_paused, next_paused);
                let _ = toggle_for_handler.set_text(if next_paused {
                    "Resume Monitoring"
                } else {
                    "Pause Monitoring"
                });
                refresh_tray_summary(app, summary_for_handler.clone());
            }
            TRAY_QUIT_ID => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    tray.build(app)?;
    refresh_tray_summary(app.handle(), today_summary);
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

    let monitor_paused = new_monitor_paused_flag();

    tauri::Builder::default()
        .manage(AppState {
            store,
            monitor_paused,
        })
        .setup(|app| {
            setup_tray(app)?;
            let state = app.state::<AppState>();
            watcher::start_background_monitor(state.store.clone(), state.monitor_paused.clone());
            show_dashboard(app.handle());
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
            commands::session_messages,
            commands::session_model_requests,
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
        assert_eq!(TRAY_ID, "codex-token-monitor");
        assert_eq!(TRAY_OPEN_DASHBOARD_ID, "open-dashboard");
        assert_eq!(TRAY_TODAY_SUMMARY_ID, "today-summary");
        assert_eq!(TRAY_TOGGLE_MONITORING_ID, "toggle-monitoring");
        assert_eq!(TRAY_QUIT_ID, "quit");
    }

    #[test]
    fn app_state_monitoring_pause_flag_defaults_to_running() {
        let paused = new_monitor_paused_flag();
        assert!(!is_monitoring_paused(&paused));
    }

    #[test]
    fn app_state_monitoring_pause_flag_toggles() {
        let paused = new_monitor_paused_flag();
        assert!(set_monitoring_paused(&paused, true));
        assert!(!set_monitoring_paused(&paused, false));
    }
}
