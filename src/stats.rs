use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub phase: String,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DayStats {
    pub date: String,
    pub pomodoros: u32,
    pub total_work_secs: u64,
    pub sessions: Vec<SessionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsData {
    pub days: HashMap<String, DayStats>,
    pub all_time_pomodoros: u32,
    pub all_time_work_secs: u64,
    pub best_streak: u32,
}

impl StatsData {
    fn stats_path() -> PathBuf {
        let base = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = base.join(".pomo-clock");
        let _ = fs::create_dir_all(&dir);
        dir.join("stats.json")
    }

    pub fn load() -> Self {
        let path = Self::stats_path();
        if path.exists() {
            let data = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::stats_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, data);
        }
    }

    pub fn record_pomodoro(&mut self, start: DateTime<Local>, end: DateTime<Local>, duration_secs: u64) {
        let date_key = end.format("%Y-%m-%d").to_string();
        let day = self.days.entry(date_key.clone()).or_insert_with(|| DayStats {
            date: date_key,
            ..Default::default()
        });

        day.pomodoros += 1;
        day.total_work_secs += duration_secs;
        day.sessions.push(SessionRecord {
            start,
            end,
            phase: "WORK".to_string(),
            duration_secs,
        });

        self.all_time_pomodoros += 1;
        self.all_time_work_secs += duration_secs;

        // Recalculate best streak
        let streak = self.current_streak();
        if streak > self.best_streak {
            self.best_streak = streak;
        }

        self.save();
    }

    pub fn today_pomodoros(&self) -> u32 {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.days.get(&today).map(|d| d.pomodoros).unwrap_or(0)
    }

    pub fn today_work_secs(&self) -> u64 {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.days.get(&today).map(|d| d.total_work_secs).unwrap_or(0)
    }

    pub fn today_sessions(&self) -> Vec<SessionRecord> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        self.days.get(&today).map(|d| d.sessions.clone()).unwrap_or_default()
    }

    pub fn current_streak(&self) -> u32 {
        let today = Local::now().date_naive();
        let mut streak = 0u32;
        let mut check_date = today;

        loop {
            let key = check_date.format("%Y-%m-%d").to_string();
            match self.days.get(&key) {
                Some(day) if day.pomodoros >= 4 => {
                    streak += 1;
                    check_date = check_date.pred_opt().unwrap_or(check_date);
                }
                _ => break,
            }
        }

        streak
    }

    pub fn week_daily_totals(&self) -> Vec<(String, u32)> {
        let today = Local::now().date_naive();
        let mut result = Vec::new();
        for i in (0..7).rev() {
            let date = today - chrono::Duration::days(i);
            let key = date.format("%Y-%m-%d").to_string();
            let short = date.format("%a").to_string();
            let count = self.days.get(&key).map(|d| d.pomodoros).unwrap_or(0);
            result.push((short, count));
        }
        result
    }

    pub fn all_time_hours(&self) -> f64 {
        self.all_time_work_secs as f64 / 3600.0
    }
}
