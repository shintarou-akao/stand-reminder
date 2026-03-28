mod modal;
mod settings;
mod sound;
mod state;
mod timer;
mod tray;

use std::sync::{Arc, Mutex};
use state::AppState;
use tauri::{AppHandle, Emitter};

#[tauri::command]
fn get_state(state: tauri::State<'_, Arc<Mutex<AppState>>>) -> state::StateSnapshot {
    state.lock().unwrap().snapshot()
}

#[tauri::command]
fn get_settings(app: AppHandle) -> settings::Settings {
    settings::load(&app)
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    settings: settings::Settings,
    state: tauri::State<'_, Arc<Mutex<AppState>>>,
) {
    settings::save(&app, &settings);
    let snapshot = {
        let mut s = state.lock().unwrap();
        s.apply_settings(&settings);
        s.reset_timer();
        s.snapshot()
    };
    let _ = app.emit("state-changed", &snapshot);
    tray::rebuild_menu(&app, &snapshot);
}

#[tauri::command]
fn preview_sound(app: AppHandle, name: String) {
    let _ = app.run_on_main_thread(move || {
        sound::play_sound(&name);
    });
}

#[tauri::command]
fn get_sound_names() -> Vec<&'static str> {
    sound::SOUND_NAMES.to_vec()
}

#[tauri::command]
fn stood_up(app: AppHandle, state: tauri::State<'_, Arc<Mutex<AppState>>>) {
    let snapshot = {
        let mut s = state.lock().unwrap();
        s.reset_timer();
        s.snapshot()
    };
    modal::hide_modal(&app);
    let _ = app.emit("state-changed", &snapshot);
    tray::rebuild_menu(&app, &snapshot);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = Arc::new(Mutex::new(AppState::from_settings(&settings::Settings::default())));

    let app = tauri::Builder::default()
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            get_state,
            get_settings,
            save_settings,
            stood_up,
            preview_sound,
            get_sound_names
        ])
        .setup(move |app| {
            // tao のデフォルトは Regular ポリシーで、applicationDidFinishLaunching 時に
            // activateIgnoringOtherApps が呼ばれ NSStatusItem メニュー初回表示と干渉する。
            // Accessory に設定することで回避する（本番は LSUIElement が担うが dev では効かない）。
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let loaded = settings::load(app.handle());
            {
                let mut s = app_state.lock().unwrap();
                s.apply_settings(&loaded);
                s.reset_timer();
            }

            tray::setup_tray(app)?;
            timer::start_timer(app.handle().clone(), app_state.clone());

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building tauri application");

    app.run(|_app, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();
        }
    });
}
