use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::state::{AppState, StateSnapshot};
use std::sync::{Arc, Mutex};

fn load_tray_icon() -> tauri::image::Image<'static> {
    tauri::include_image!("icons/tray.png")
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let state = app.state::<Arc<Mutex<AppState>>>();
    let snapshot = state.lock().unwrap().snapshot();
    let menu = build_menu(app, &snapshot)?;

    let icon = load_tray_icon();

    TrayIconBuilder::with_id("main-tray")
        .icon(icon)
        .icon_as_template(true)
        .menu(&menu)
        .on_menu_event(|app, event| handle_menu_event(app, event.id().as_ref()))
        .build(app)?;

    Ok(())
}


fn notify_time_label(remaining_secs: u64) -> String {
    use chrono::Local;
    let notify_at = Local::now() + chrono::Duration::seconds(remaining_secs as i64);
    notify_at.format("%H:%M").to_string()
}

fn build_menu(app: &AppHandle, snapshot: &StateSnapshot) -> tauri::Result<Menu<tauri::Wry>> {
    let label = match (snapshot.timer_running, snapshot.timer_remaining_secs) {
        (true, _) => format!("Next: {}", notify_time_label(snapshot.timer_remaining_secs)),
        (false, 0) => "No reminders".to_string(),
        (false, _) => "Notifying...".to_string(),
    };
    let status = MenuItem::with_id(app, "status", &label, false, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    Menu::with_items(app, &[&status, &sep, &settings, &quit])
}

/// メニュー構造を再構築する（モード切替・タイマー終了など状態変化時のみ呼ぶ）
pub fn rebuild_menu(app: &AppHandle, snapshot: &StateSnapshot) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        if let Ok(menu) = build_menu(app, snapshot) {
            let _ = tray.set_menu(Some(menu));
        }
        update_title_inner(&tray, snapshot);
    }
}

/// トレイタイトルだけ更新する（分が変わった時のみ呼ぶ・開いているメニューを閉じない）
pub fn update_title(app: &AppHandle, snapshot: &StateSnapshot) {
    if let Some(tray) = app.tray_by_id("main-tray") {
        update_title_inner(&tray, snapshot);
    }
}

#[cfg(target_os = "macos")]
fn update_title_inner(tray: &tauri::tray::TrayIcon, _snapshot: &StateSnapshot) {
    let _ = tray.set_title(None::<&str>);
}

#[cfg(not(target_os = "macos"))]
fn update_title_inner(_tray: &tauri::tray::TrayIcon, _snapshot: &StateSnapshot) {}

fn handle_menu_event(app: &AppHandle, event_id: &str) {
    use crate::modal;

    match event_id {
        "settings" => {
            let _ = modal::show_settings(app);
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}
