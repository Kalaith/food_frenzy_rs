//! Soft day/shift framing: a real-time day clock, per-day stats for the
//! end-of-day ledger, and the pause-the-world summary gate between days.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DayStats {
    pub cash_earned: i64,
    pub renown_earned: i64,
    pub guests_served: u32,
    pub guests_lost: u32,
    pub meat_gained: i64,
    pub guests_processed: u32,
    pub fresh_dishes: u32,
    pub best_combo: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayCycle {
    pub day: u32,
    pub elapsed_ms: f32,
    /// True between the day ending and the player opening the next one; the
    /// world pauses and the ledger overlay is up.
    pub summary_pending: bool,
    /// Whether this day's dining event has already fired.
    pub event_fired: bool,
    pub stats: DayStats,
}

impl Default for DayCycle {
    fn default() -> Self {
        Self {
            day: 1,
            elapsed_ms: 0.0,
            summary_pending: false,
            event_fired: false,
            stats: DayStats::default(),
        }
    }
}

impl DayCycle {
    /// Advance the clock; returns true on the tick the day ends.
    pub fn update(&mut self, dt_ms: f32, day_length_ms: f32) -> bool {
        if self.summary_pending {
            return false;
        }
        self.elapsed_ms += dt_ms.max(0.0);
        if self.elapsed_ms >= day_length_ms.max(1_000.0) {
            self.summary_pending = true;
            return true;
        }
        false
    }

    pub fn start_next_day(&mut self) {
        self.day = self.day.saturating_add(1);
        self.elapsed_ms = 0.0;
        self.summary_pending = false;
        self.event_fired = false;
        self.stats = DayStats::default();
    }

    /// 0..1 progress through the current day (for the clock UI).
    pub fn day_progress(&self, day_length_ms: f32) -> f32 {
        (self.elapsed_ms / day_length_ms.max(1.0)).clamp(0.0, 1.0)
    }

    pub fn record_combo(&mut self, combo: u32) {
        self.stats.best_combo = self.stats.best_combo.max(combo);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_ends_once_and_waits_for_the_summary() {
        let mut cycle = DayCycle::default();
        assert!(!cycle.update(1_000.0, 10_000.0));
        assert!(cycle.update(9_500.0, 10_000.0), "day should end");
        assert!(cycle.summary_pending);
        assert!(
            !cycle.update(60_000.0, 10_000.0),
            "clock holds while the ledger is up"
        );
        assert_eq!(cycle.day, 1);
    }

    #[test]
    fn next_day_resets_the_ledger() {
        let mut cycle = DayCycle::default();
        cycle.stats.cash_earned = 120;
        cycle.record_combo(7);
        cycle.update(20_000.0, 10_000.0);
        cycle.start_next_day();
        assert_eq!(cycle.day, 2);
        assert_eq!(cycle.stats.cash_earned, 0);
        assert_eq!(cycle.stats.best_combo, 0);
        assert!(!cycle.summary_pending);
        assert!(!cycle.event_fired);
    }
}
