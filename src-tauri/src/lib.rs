mod modal;
mod settings;
mod sound;
mod state;
mod timer;
mod tray;

use std::sync::{Arc, Mutex};
use state::AppState;
use tauri::{AppHandle, Emitter, Manager};

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
            let loaded = settings::load(app.handle());
            {
                let mut s = app_state.lock().unwrap();
                s.apply_settings(&loaded);
                s.reset_timer();
            }

            // トレイ作成・タイマー開始は RunEvent::Ready 後に遅延する。
            // tao が applicationDidFinishLaunching で Regular ポリシー + activateIgnoringOtherApps
            // を実行するため、その直後に Accessory へ切り替え、activation が落ち着いた
            // 次のイベントループ反復でトレイを作成することで初回メニュー即閉じを回避する。
            let handle = app.handle().clone();
            let state = app.state::<Arc<Mutex<AppState>>>().inner().clone();
            app.handle().run_on_main_thread(move || {
                #[cfg(target_os = "macos")]
                {
                    use objc2::MainThreadMarker;
                    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
                    unsafe {
                        let mtm = MainThreadMarker::new_unchecked();
                        NSApplication::sharedApplication(mtm)
                            .setActivationPolicy(NSApplicationActivationPolicy::Accessory);
                    }
                }

                tray::setup_tray(&handle).expect("failed to setup tray");
                timer::start_timer(handle, state);
            })?;

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
