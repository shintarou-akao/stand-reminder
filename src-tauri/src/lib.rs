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

    tauri::Builder::default()
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
            let loaded = settings::load(app.handle());
            {
                let mut s = app_state.lock().unwrap();
                s.apply_settings(&loaded);
                s.reset_timer();
            }

            // macOSPrivateApi が activation policy を Regular に変えてしまうため
            // Accessory に戻す（Dock非表示・メニューバー名変化なし・メニュー表示可能）
            #[cfg(target_os = "macos")]
            {
                use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
                unsafe {
                    let mtm = objc2::MainThreadMarker::new_unchecked();
                    NSApplication::sharedApplication(mtm)
                        .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
                }
            }

            tray::setup_tray(app)?;
            timer::start_timer(app.handle().clone(), app_state.clone());

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building tauri application")
        .run(|_app, event| {
            match event {
                tauri::RunEvent::Ready => {
                    // setup 後に Tauri が policy を上書きする場合に備えて再設定
                    #[cfg(target_os = "macos")]
                    {
                        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
                        unsafe {
                            let mtm = objc2::MainThreadMarker::new_unchecked();
                            NSApplication::sharedApplication(mtm)
                                .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
                        }
                    }
                }
                tauri::RunEvent::ExitRequested { api, .. } => {
                    api.prevent_exit();
                }
                _ => {}
            }
        });
}
