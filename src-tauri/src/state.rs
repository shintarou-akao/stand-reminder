use serde::Serialize;
use crate::settings::{ReminderMode, Settings};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateSnapshot {
    pub timer_remaining_secs: u64,
    pub timer_running: bool,
}

pub struct AppState {
    pub timer_remaining_secs: u64,
    pub timer_running: bool,
    pub reminder_mode: ReminderMode,
    pub sit_duration_secs: u64,
    pub specific_times: Vec<String>,
    pub sound_enabled: bool,
    pub sound_name: String,
}

impl AppState {
    pub fn from_settings(settings: &Settings) -> Self {
        let sit = settings.remind_interval_mins * 60;
        let mut s = Self {
            timer_remaining_secs: sit,
            timer_running: true,
            reminder_mode: settings.reminder_mode.clone(),
            sit_duration_secs: sit,
            specific_times: settings.specific_times.clone(),
            sound_enabled: settings.sound_enabled,
            sound_name: settings.sound_name.clone(),
        };
        s.reset_timer();
        s
    }

    pub fn apply_settings(&mut self, settings: &Settings) {
        self.reminder_mode = settings.reminder_mode.clone();
        self.sit_duration_secs = settings.remind_interval_mins * 60;
        self.specific_times = settings.specific_times.clone();
        self.sound_enabled = settings.sound_enabled;
        self.sound_name = settings.sound_name.clone();
    }

    pub fn snapshot(&self) -> StateSnapshot {
        StateSnapshot {
            timer_remaining_secs: self.timer_remaining_secs,
            timer_running: self.timer_running,
        }
    }

    pub fn reset_timer(&mut self) {
        match self.reminder_mode {
            ReminderMode::Interval => {
                self.timer_remaining_secs = self.sit_duration_secs;
                self.timer_running = true;
            }
            ReminderMode::SpecificTimes => {
                if let Some(secs) = secs_until_next_specific(&self.specific_times) {
                    self.timer_remaining_secs = secs;
                    self.timer_running = true;
                } else {
                    // 時刻未設定 → タイマー停止
                    self.timer_remaining_secs = 0;
                    self.timer_running = false;
                }
            }
        }
    }

    pub fn reset_on_wake(&mut self) {
        self.reset_timer();
    }
}

/// 指定時刻リストから現在時刻以降の最も近い時刻までの秒数を返す
fn secs_until_next_specific(times: &[String]) -> Option<u64> {
    use chrono::{Local, NaiveTime, Timelike};

    let now = Local::now();
    let now_time = now.time();

    let mut parsed: Vec<NaiveTime> = times
        .iter()
        .filter_map(|t| NaiveTime::parse_from_str(t, "%H:%M").ok())
        .collect();
    parsed.sort();

    if parsed.is_empty() {
        return None;
    }

    // 今日の残り時刻から最も近いものを探す（整数秒同士で計算してサブ秒誤差を防ぐ）
    let now_secs = now_time.num_seconds_from_midnight() as u64;
    if let Some(next) = parsed.iter().find(|&&t| t > now_time) {
        let next_secs = next.num_seconds_from_midnight() as u64;
        return Some(next_secs - now_secs);
    }

    // すべて過ぎていたら明日の最初の時刻
    let first = parsed[0];
    let secs_until_midnight =
        86400 - now_time.num_seconds_from_midnight() as u64;
    let secs_from_midnight = first.num_seconds_from_midnight() as u64;
    Some(secs_until_midnight + secs_from_midnight)
}
