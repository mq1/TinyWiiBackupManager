// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

/// Returned by [`TaskControl::checkpoint`] when the task was cancelled; the worker stops and
/// cleans up what it half-wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("cancelled")
    }
}

impl std::error::Error for Cancelled {}

/// Shared between the task manager and the worker doing the task: the manager sets cancel and
/// pause, the worker calls `checkpoint` between chunks of work.
#[derive(Debug, Clone, Default)]
pub struct TaskControl {
    inner: Arc<Flags>,
}

#[derive(Debug, Default)]
struct Flags {
    cancelled: AtomicBool,
    paused: AtomicBool,
}

impl TaskControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.inner.cancelled.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::Relaxed)
    }

    pub fn set_paused(&self, paused: bool) {
        self.inner.paused.store(paused, Ordering::Relaxed);
    }

    pub fn is_paused(&self) -> bool {
        self.inner.paused.load(Ordering::Relaxed)
    }

    /// For blocking workers: waits while the task is paused, and fails once it is cancelled.
    pub fn checkpoint(&self) -> Result<(), Cancelled> {
        while self.is_paused() && !self.is_cancelled() {
            thread::sleep(Duration::from_millis(100));
        }

        if self.is_cancelled() {
            Err(Cancelled)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoint_passes_until_cancelled() {
        let control = TaskControl::new();
        let worker = control.clone();
        assert_eq!(worker.checkpoint(), Ok(()));
        control.cancel();
        assert_eq!(worker.checkpoint(), Err(Cancelled));
    }

    #[test]
    fn checkpoint_waits_while_paused() {
        let control = TaskControl::new();
        control.set_paused(true);

        let worker = control.clone();
        let handle = thread::spawn(move || worker.checkpoint());

        thread::sleep(Duration::from_millis(250));
        assert!(!handle.is_finished());

        control.set_paused(false);
        assert_eq!(handle.join().unwrap(), Ok(()));
    }

    #[test]
    fn cancel_ends_a_pause() {
        let control = TaskControl::new();
        control.set_paused(true);
        let worker = control.clone();
        let handle = thread::spawn(move || worker.checkpoint());
        control.cancel();
        assert_eq!(handle.join().unwrap(), Err(Cancelled));
    }
}
