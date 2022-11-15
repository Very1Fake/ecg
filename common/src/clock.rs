use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use spin_sleep::sleep;

use crate::span;

/// Clock tries to keep tick a constant time
pub struct Clock {
    /// Target tick duration
    pub target: Duration,
    /// Last tick time
    last: Instant,
    /// Last tick duration
    last_dur: Duration,

    // Statistics related
    /// Statistics store
    stats: ClockStats,
    /// Tick durations history
    tick_durs: VecDeque<f32>,
    /// Tick busy durations history
    tick_busy_durs: VecDeque<f32>,
}

impl Clock {
    pub const HISTORY_LENGTH: usize = 100;

    pub fn new(target: Duration) -> Self {
        Self {
            target,
            last: Instant::now(),
            last_dur: target,
            stats: ClockStats::new(),
            tick_durs: VecDeque::with_capacity(Self::HISTORY_LENGTH),
            tick_busy_durs: VecDeque::with_capacity(Self::HISTORY_LENGTH),
        }
    }

    pub fn stats(&self) -> ClockStats {
        self.stats.clone()
    }

    pub fn tps_to_duration(tps: u32) -> Duration {
        Duration::from_secs_f64(1.0 / tps as f64)
    }

    /// Get last tick duration
    pub fn duration(&self) -> Duration {
        self.last_dur
    }

    pub fn tick(&mut self) {
        span!(_guard, "tick", "Clock::tick");

        // Current system time
        let now = Instant::now();
        // Duration between last end time and current tick start time.
        // Duration of frame time
        let busy = now.duration_since(self.last);

        // Update stats
        self.stats.update(&self.tick_durs, &self.tick_busy_durs);

        // Sleep if current tick duration is not negative
        if let Some(sleep_dur) = self.target.checked_sub(busy) {
            sleep(sleep_dur);
        }

        // Time after sleep
        let after = Instant::now();
        // Save duration of current tick
        self.last_dur = after.duration_since(self.last);

        if self.tick_durs.len() >= Self::HISTORY_LENGTH {
            self.tick_durs.pop_front();
        }
        if self.tick_busy_durs.len() >= Self::HISTORY_LENGTH {
            self.tick_busy_durs.pop_front();
        }

        // Save current tick total duration to history
        self.tick_durs.push_back(self.last_dur.as_secs_f32());
        // Save current tick busy duration to history
        self.tick_busy_durs.push_back(busy.as_secs_f32());

        // Maintain total time counter
        self.stats.total += self.last_dur;
        // Save current tick time
        self.last = after;
    }
}

// TODO: Add percentiles (50, 90, 95, 99)
#[derive(Clone)]
pub struct ClockStats {
    /// Total clock duration
    pub total: Duration,
    /// Average tick duration
    // FIX: Debug (strange behavior)
    pub avg_tick_dur: Duration,
    /// Average ticks per second
    pub avg_tps: f32,
}

impl ClockStats {
    pub const fn new() -> Self {
        Self {
            total: Duration::ZERO,
            avg_tick_dur: Duration::ZERO,
            avg_tps: 0.0,
        }
    }

    pub fn update(&mut self, tick_durs: &VecDeque<f32>, tick_busy_durs: &VecDeque<f32>) {
        self.avg_tick_dur = Duration::from_secs_f32(
            tick_busy_durs.iter().sum::<f32>() / tick_busy_durs.len().max(1) as f32,
        );
        self.avg_tps = 1.0 / (tick_durs.iter().sum::<f32>() / tick_durs.len().max(1) as f32);
    }
}
