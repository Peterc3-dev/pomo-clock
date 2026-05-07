use crate::stats::StatsData;
use chrono::{DateTime, Local};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn label(&self) -> &'static str {
        match self {
            Phase::Work => "WORK",
            Phase::ShortBreak => "SHORT BREAK",
            Phase::LongBreak => "LONG BREAK",
        }
    }
}

pub struct App {
    // Config
    pub work_secs: u64,
    pub short_break_secs: u64,
    pub long_break_secs: u64,
    pub sessions_before_long: u32,
    pub auto_start: bool,
    pub notify_cmd: Option<String>,

    // Timer state
    pub phase: Phase,
    pub remaining_secs: u64,
    pub total_phase_secs: u64,
    pub running: bool,
    pub completed_work_sessions: u32,

    // Tracking
    pub phase_start: Option<DateTime<Local>>,
    pub last_tick: Option<Instant>,
    pub accumulated_ms: u64,

    // Stats
    pub stats: StatsData,

    // UI
    pub show_stats: bool,
}

impl App {
    pub fn new(
        work_min: u64,
        short_break_min: u64,
        long_break_min: u64,
        sessions_before_long: u32,
        auto_start: bool,
        notify_cmd: Option<String>,
    ) -> Self {
        let work_secs = work_min * 60;
        let stats = StatsData::load();
        Self {
            work_secs,
            short_break_secs: short_break_min * 60,
            long_break_secs: long_break_min * 60,
            sessions_before_long,
            auto_start,
            notify_cmd,
            phase: Phase::Work,
            remaining_secs: work_secs,
            total_phase_secs: work_secs,
            running: false,
            completed_work_sessions: 0,
            phase_start: None,
            last_tick: None,
            accumulated_ms: 0,
            stats,
            show_stats: false,
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.running {
            self.running = false;
            self.last_tick = None;
        } else {
            self.running = true;
            self.last_tick = Some(Instant::now());
            if self.phase_start.is_none() {
                self.phase_start = Some(Local::now());
            }
        }
    }

    pub fn reset_current(&mut self) {
        self.remaining_secs = self.total_phase_secs;
        self.running = false;
        self.last_tick = None;
        self.accumulated_ms = 0;
        self.phase_start = None;
    }

    pub fn skip_phase(&mut self) {
        self.complete_phase();
    }

    pub fn adjust_time(&mut self, delta_secs: i64) {
        let new_remaining = self.remaining_secs as i64 + delta_secs;
        if new_remaining > 0 {
            self.remaining_secs = new_remaining as u64;
            let new_total = self.total_phase_secs as i64 + delta_secs;
            if new_total > 0 {
                self.total_phase_secs = new_total as u64;
            }
        }
    }

    pub fn toggle_stats_view(&mut self) {
        self.show_stats = !self.show_stats;
    }

    pub fn tick(&mut self) {
        if !self.running || self.remaining_secs == 0 {
            return;
        }

        let now = Instant::now();
        if let Some(last) = self.last_tick {
            self.accumulated_ms += now.duration_since(last).as_millis() as u64;
            // Decrement one second at a time
            while self.accumulated_ms >= 1000 && self.remaining_secs > 0 {
                self.accumulated_ms -= 1000;
                self.remaining_secs -= 1;
            }
        }
        self.last_tick = Some(now);

        if self.remaining_secs == 0 {
            self.complete_phase();
        }
    }

    fn complete_phase(&mut self) {
        // Ring terminal bell
        print!("\x07");

        // Run notify command
        if let Some(ref cmd) = self.notify_cmd {
            let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
        }

        if self.phase == Phase::Work {
            let end = Local::now();
            let start = self.phase_start.unwrap_or(end);
            self.stats.record_pomodoro(start, end, self.total_phase_secs);
            self.completed_work_sessions += 1;
        }

        // Advance to next phase
        let next_phase = match self.phase {
            Phase::Work => {
                if self.completed_work_sessions > 0
                    && self.completed_work_sessions % self.sessions_before_long == 0
                {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };

        self.phase = next_phase;
        self.total_phase_secs = match next_phase {
            Phase::Work => self.work_secs,
            Phase::ShortBreak => self.short_break_secs,
            Phase::LongBreak => self.long_break_secs,
        };
        self.remaining_secs = self.total_phase_secs;
        self.accumulated_ms = 0;
        self.phase_start = None;
        self.last_tick = None;

        if self.auto_start {
            self.running = true;
            self.last_tick = Some(Instant::now());
            self.phase_start = Some(Local::now());
        } else {
            self.running = false;
        }
    }

    pub fn progress(&self) -> f64 {
        if self.total_phase_secs == 0 {
            return 0.0;
        }
        let elapsed = self.total_phase_secs.saturating_sub(self.remaining_secs);
        elapsed as f64 / self.total_phase_secs as f64
    }

    pub fn minutes(&self) -> u64 {
        self.remaining_secs / 60
    }

    pub fn seconds(&self) -> u64 {
        self.remaining_secs % 60
    }
}
