//! An easy-to-use timer library.

extern crate chrono;
use chrono::{DateTime, Utc};

use std::cmp;
use std::time::Duration;

/// A timer.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Timer {
    /// The timer has not been started yet.
    NotStarted,
    /// The timer has been started.
    Started {
        /// The time when the timer was last marked.
        last_marked: DateTime<Utc>,
    },
}

impl Timer {
    /// Creates a new timer.
    pub fn new() -> Self {
        Timer::NotStarted
    }

    /// Marks a new instant, returning the number of seconds since the last mark.
    ///
    /// The very first mark will return a delta time of `0.0`. Because of this,
    /// any time between timer creation and the first mark will have no affect
    /// on the delta times returned.
    pub fn mark(&mut self) -> Duration {
        Duration::from_millis(self.mark_millis())
    }

    /// Marks a new instant, returning the number of milliseconds since the last mark.
    pub fn mark_millis(&mut self) -> u64 {
        match *self {
            // On the very first mark, start timer and return 0 until next call.
            Timer::NotStarted => {
                *self = Timer::Started { last_marked: Utc::now() };
                0
            },
            Timer::Started { ref mut last_marked } => {
                let now = Utc::now();

                let delta_ms = cmp::max(now.signed_duration_since(*last_marked).num_milliseconds(), 0);
                *last_marked = now;

                // Negative times count as zero.
                cmp::max(delta_ms, 0) as u64
            },
        }
    }

    /// Marks a new instant, returning the number of seconds since the last mark.
    ///
    /// The value returned is fractional - if 1 millisecond has passed, then
    /// `0.001` will be returned.
    pub fn mark_secs(&mut self) -> f64 {
        self.mark_millis() as f64 / 1000.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn first_mark_is_zero_seconds() {
        assert_eq!(0.0, Timer::new().mark());
    }
}

