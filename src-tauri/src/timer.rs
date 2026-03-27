use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::time::MissedTickBehavior;

use crate::modal;
use crate::sound;
use crate::state::AppState;
use crate::tray;

pub fn start_timer(app: AppHandle, state: Arc<Mutex<AppState>>) {
    // 起動直後の不要な update_title 呼び出しを防ぐため、現在値で初期化
    let initial_mins = {
        let s = state.lock().unwrap();
        if s.timer_running {
            (s.timer_remaining_secs + 59) / 60
        } else {
            u64::MAX
        }
    };

    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut last_tick = Instant::now();
        let mut last_displayed_mins = initial_mins;

        loop {
            interval.tick().await;

            let now = Instant::now();
            let elapsed = now.duration_since(last_tick);
            last_tick = now;

            // スリープ復帰検知: 10秒以上経過していたらリセット
            if elapsed.as_secs() > 10 {
                let snapshot = {
                    let mut s = state.lock().unwrap();
                    s.reset_on_wake();
                    s.snapshot()
                };
                modal::hide_modal(&app);
                let _ = app.emit("state-changed", &snapshot);
                tray::rebuild_menu(&app, &snapshot);
                continue;
            }

            let (show_modal, snapshot) = {
                let mut s = state.lock().unwrap();

                if !s.timer_running {
                    (false, s.snapshot())
                } else if s.timer_remaining_secs > 0 {
                    s.timer_remaining_secs -= 1;
                    (false, s.snapshot())
                } else {
                    s.timer_running = false;
                    (true, s.snapshot())
                }
            };

            let _ = app.emit("state-changed", &snapshot);

            if show_modal {
                tray::rebuild_menu(&app, &snapshot);
                last_displayed_mins = u64::MAX;
                let (sound_enabled, sound_name) = {
                    let s = state.lock().unwrap();
                    (s.sound_enabled, s.sound_name.clone())
                };
                if sound_enabled {
                    let app_clone = app.clone();
                    let _ = app_clone.run_on_main_thread(move || {
                        sound::play_sound(&sound_name);
                    });
                }
                let _ = modal::show_modal(&app);
            } else {
                let current_mins = (snapshot.timer_remaining_secs + 59) / 60;
                if current_mins != last_displayed_mins {
                    last_displayed_mins = current_mins;
                    tray::update_title(&app, &snapshot);
                }
            }
        }
    });
}
