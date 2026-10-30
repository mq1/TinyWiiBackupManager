// SPDX-FileCopyrightText: 2026 Ultimate Game Backup Manager contributors
// SPDX-License-Identifier: GPL-3.0-only

use smol::{channel::Sender, stream::Stream};
use std::time::{Duration, Instant};

/// What a worker tells the task manager about its task.
#[derive(Debug, Clone)]
pub enum WorkerEvent {
    Progress {
        done_bytes: u64,
        total_bytes: Option<u64>,
        message: String,
    },
    /// A message for the user, or the reason it failed.
    Finished(Result<String, String>),
}

/// The worker's side of the channel. Progress is sent at most every [`Reporter::INTERVAL`], so a
/// fast copy doesn't flood the UI with updates.
pub struct Reporter {
    tx: Sender<WorkerEvent>,
    last: Option<Instant>,
}

impl Reporter {
    pub const INTERVAL: Duration = Duration::from_millis(200);

    pub fn progress(
        &mut self,
        done_bytes: u64,
        total_bytes: Option<u64>,
        message: impl FnOnce() -> String,
    ) {
        let now = Instant::now();
        if self
            .last
            .is_some_and(|last| now.duration_since(last) < Self::INTERVAL)
        {
            return;
        }

        self.last = Some(now);
        // a full channel means the UI hasn't shown the last update yet; this one can go
        let _ = self.tx.try_send(WorkerEvent::Progress {
            done_bytes,
            total_bytes,
            message: message(),
        });
    }
}

/// Runs a blocking worker on its own thread and streams what it reports, ending with
/// [`WorkerEvent::Finished`].
pub fn spawn_blocking(
    work: impl FnOnce(&mut Reporter) -> Result<String, String> + Send + 'static,
) -> impl Stream<Item = WorkerEvent> {
    let (tx, rx) = smol::channel::bounded(1);

    std::thread::spawn(move || {
        let mut reporter = Reporter {
            tx: tx.clone(),
            last: None,
        };
        let result = work(&mut reporter);
        let _ = tx.send_blocking(WorkerEvent::Finished(result));
    });

    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::stream::StreamExt;

    #[test]
    fn reports_progress_then_the_result() {
        let events: Vec<WorkerEvent> = smol::block_on(
            spawn_blocking(|reporter| {
                reporter.progress(1, Some(2), || "first".into());
                // within the interval: dropped
                reporter.progress(2, Some(2), || "second".into());
                Ok("done".into())
            })
            .collect(),
        );

        assert!(matches!(
            &events[..],
            [
                WorkerEvent::Progress { done_bytes: 1, .. },
                WorkerEvent::Finished(Ok(msg)),
            ] if msg == "done"
        ));
    }
}
