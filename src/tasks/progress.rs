// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

use size::Size;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

/// What a running task reports: how far it is, and a line saying what it does right now.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Progress {
    pub done_bytes: u64,
    /// `None` while the total isn't known yet (e.g. a download without a length).
    pub total_bytes: Option<u64>,
    pub message: String,
    /// Bytes per second, from the [`SpeedMeter`].
    pub speed: Option<f64>,
}

impl Progress {
    /// 0.0 to 1.0, or `None` while the total isn't known.
    pub fn fraction(&self) -> Option<f32> {
        match self.total_bytes {
            Some(0) => Some(1.0),
            Some(total) => Some((self.done_bytes.min(total) as f64 / total as f64) as f32),
            None => None,
        }
    }

    pub fn time_left(&self) -> Option<Duration> {
        let total = self.total_bytes?;
        let speed = self.speed.filter(|s| *s > 0.0)?;
        let left = total.saturating_sub(self.done_bytes) as f64;
        Some(Duration::from_secs_f64(left / speed))
    }

    /// One line for the user: "1.20 GiB of 4.37 GiB · 35.0 MiB/s · 2 min left".
    pub fn summary(&self) -> String {
        let mut parts = vec![match self.total_bytes {
            Some(total) => format!(
                "{} of {}",
                Size::from_bytes(self.done_bytes.min(total)),
                Size::from_bytes(total)
            ),
            None => Size::from_bytes(self.done_bytes).to_string(),
        }];

        if let Some(speed) = self.speed.filter(|s| *s > 0.0) {
            parts.push(format!("{}/s", Size::from_bytes(speed as u64)));
        }

        if let Some(left) = self.time_left() {
            parts.push(format!("{} left", format_duration(left)));
        }

        parts.join(" · ")
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    match secs {
        0..60 => format!("{secs} s"),
        60..3600 => format!("{} min", secs.div_ceil(60)),
        _ => format!("{} h {} min", secs / 3600, secs % 3600 / 60),
    }
}

/// Speed over the last few seconds, so a short stall or burst doesn't make the time left jump.
#[derive(Debug, Clone)]
pub struct SpeedMeter {
    window: Duration,
    samples: VecDeque<(Instant, u64)>,
}

impl Default for SpeedMeter {
    fn default() -> Self {
        Self::new(Duration::from_secs(5))
    }
}

impl SpeedMeter {
    pub fn new(window: Duration) -> Self {
        Self {
            window,
            samples: VecDeque::new(),
        }
    }

    /// Records that `done_bytes` were done at `now`, and returns the speed in bytes per second.
    pub fn record(&mut self, now: Instant, done_bytes: u64) -> Option<f64> {
        self.samples.push_back((now, done_bytes));

        while let Some(&(t, _)) = self.samples.front() {
            if now.duration_since(t) > self.window && self.samples.len() > 2 {
                self.samples.pop_front();
            } else {
                break;
            }
        }

        let &(first_t, first_bytes) = self.samples.front()?;
        let secs = now.duration_since(first_t).as_secs_f64();
        (secs > 0.0).then(|| done_bytes.saturating_sub(first_bytes) as f64 / secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fraction_and_time_left() {
        let p = Progress {
            done_bytes: 25,
            total_bytes: Some(100),
            speed: Some(5.0),
            ..Default::default()
        };
        assert_eq!(p.fraction(), Some(0.25));
        assert_eq!(p.time_left(), Some(Duration::from_secs(15)));

        let unknown = Progress::default();
        assert_eq!(unknown.fraction(), None);
        assert_eq!(unknown.time_left(), None);
    }

    #[test]
    fn summary_line() {
        let p = Progress {
            done_bytes: 1024 * 1024,
            total_bytes: Some(4 * 1024 * 1024),
            speed: Some(1024.0 * 1024.0),
            ..Default::default()
        };
        assert_eq!(p.summary(), "1.00 MiB of 4.00 MiB · 1.00 MiB/s · 3 s left");
        assert_eq!(format_duration(Duration::from_secs(61)), "2 min");
        assert_eq!(format_duration(Duration::from_secs(3725)), "1 h 2 min");
    }

    #[test]
    fn speed_over_the_window() {
        let start = Instant::now();
        let mut meter = SpeedMeter::new(Duration::from_secs(5));
        assert_eq!(meter.record(start, 0), None);

        let speed = meter.record(start + Duration::from_secs(2), 200).unwrap();
        assert!((speed - 100.0).abs() < 1e-9);

        // after the window, the old samples no longer count: 10 s later only the last 5 s do
        for s in 3..=12 {
            meter.record(start + Duration::from_secs(s), 200 + (s - 2) * 10);
        }
        let speed = meter.record(start + Duration::from_secs(12), 300).unwrap();
        assert!((speed - 10.0).abs() < 1e-9, "{speed}");
    }
}
