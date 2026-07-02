//! UI animation clock.
//!
//! Every wall-clock read in the UI layer goes through this module so that
//! snapshot tests can freeze time and render byte-identical frames. Spinners,
//! pulses, blink phases, and countdowns all derive from these functions —
//! never call `SystemTime::now()` / `Utc::now()` directly from rendering
//! code.

use chrono::{DateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
thread_local! {
    static FROZEN_MILLIS: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
fn frozen() -> Option<u64> {
    FROZEN_MILLIS.with(|c| c.get())
}

/// Milliseconds since the UNIX epoch. Drives spinner frames, pulse phases,
/// and blink cycles.
pub fn now_millis() -> u128 {
    #[cfg(test)]
    if let Some(ms) = frozen() {
        return ms as u128;
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Seconds since the UNIX epoch. Drives countdown displays that compare
/// against stored timestamps.
pub fn now_secs() -> i64 {
    (now_millis() / 1000) as i64
}

/// Current UTC time. Drives daily-rotation countdowns and mission timers.
pub fn now_utc() -> DateTime<Utc> {
    #[cfg(test)]
    if let Some(ms) = frozen() {
        if let Some(dt) = DateTime::from_timestamp_millis(ms as i64) {
            return dt;
        }
    }
    Utc::now()
}

/// Freezes the UI clock on the current thread for the lifetime of the
/// returned guard. Rendering the same state twice under a frozen clock
/// produces identical frames — the foundation of the snapshot tests.
#[cfg(test)]
pub fn freeze_at_millis(ms: u64) -> FrozenClockGuard {
    FROZEN_MILLIS.with(|c| c.set(Some(ms)));
    FrozenClockGuard
}

#[cfg(test)]
pub struct FrozenClockGuard;

#[cfg(test)]
impl Drop for FrozenClockGuard {
    fn drop(&mut self) {
        FROZEN_MILLIS.with(|c| c.set(None));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freeze_pins_all_clock_reads() {
        let _guard = freeze_at_millis(1_750_000_000_123);
        assert_eq!(now_millis(), 1_750_000_000_123);
        assert_eq!(now_secs(), 1_750_000_000);
        assert_eq!(now_utc().timestamp_millis(), 1_750_000_000_123);
    }

    #[test]
    fn test_clock_thaws_when_guard_drops() {
        {
            let _guard = freeze_at_millis(1000);
            assert_eq!(now_millis(), 1000);
        }
        // A thawed clock reads real time again, far past the frozen value.
        assert!(now_millis() > 1_000_000);
    }
}
