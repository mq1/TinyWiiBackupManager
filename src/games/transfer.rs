// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

//! Bounded read/prepare -> write pipeline shared by import and export.

use super::disc_reader::{DiscCancellation, DiscCancelled, error_contains};
use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{self, Read, Seek, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, SyncSender, TrySendError},
    },
    time::Duration,
};

pub const BLOCK_SIZE: usize = 1024 * 1024;
// 14 queued + one being written + one pending producer block = 16 MiB.
const QUEUE_BLOCKS: usize = 14;
const POLL: Duration = Duration::from_millis(5);

enum Message {
    Data(Vec<u8>),
    Flush(SyncSender<()>),
}

#[derive(Debug)]
struct PipelineStopped;
impl std::fmt::Display for PipelineStopped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Transfer writer stopped")
    }
}
impl std::error::Error for PipelineStopped {}

struct PipelineWriter<'a, C> {
    sender: SyncSender<Message>,
    cancel: &'a C,
    abort: Arc<AtomicBool>,
    finished: bool,
}

impl<C: DiscCancellation> PipelineWriter<'_, C> {
    fn send(&self, mut message: Message) -> io::Result<()> {
        loop {
            self.cancel.check()?;
            match self.sender.try_send(message) {
                Ok(()) => return Ok(()),
                Err(TrySendError::Full(returned)) => {
                    message = returned;
                    std::thread::sleep(POLL);
                }
                Err(TrySendError::Disconnected(_)) => {
                    return Err(io::Error::other(PipelineStopped));
                }
            }
        }
    }
}

impl<C: DiscCancellation> Write for PipelineWriter<'_, C> {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.cancel.check()?;
        let count = data.len().min(BLOCK_SIZE);
        if count > 0 {
            self.send(Message::Data(data[..count].to_vec()))?;
        }
        Ok(count)
    }

    fn flush(&mut self) -> io::Result<()> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.send(Message::Flush(tx))?;
        loop {
            self.cancel.check()?;
            match rx.recv_timeout(POLL) {
                Ok(()) => return Ok(()),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return Err(io::Error::other(PipelineStopped));
                }
            }
        }
    }
}

impl<C> Drop for PipelineWriter<'_, C> {
    fn drop(&mut self) {
        // Also stops and joins the writer when the producer unwinds.
        if !self.finished {
            self.abort.store(true, Ordering::Release);
        }
    }
}

/// Returns ownership of the writer even on failure so callers can explicitly
/// discard staged files. Every path joins the writer before returning/unwinding.
pub fn transfer<W: Write + Send, C: DiscCancellation, T>(
    mut writer: W,
    fast: bool,
    cancel: &C,
    produce: impl FnOnce(&mut dyn Write) -> Result<T>,
) -> (W, Result<T>) {
    if !fast {
        let result = (|| {
            cancel.check()?;
            let value = produce(&mut writer)?;
            cancel.check()?;
            writer.flush()?;
            cancel.check()?;
            Ok(value)
        })();
        return (writer, result);
    }
    std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::sync_channel(QUEUE_BLOCKS);
        let abort = Arc::new(AtomicBool::new(false));
        let writer_abort = abort.clone();
        let worker = scope.spawn(move || {
            let result =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> io::Result<()> {
                    loop {
                        if writer_abort.load(Ordering::Acquire) {
                            return Ok(());
                        }
                        cancel.check()?;
                        match receiver.recv_timeout(POLL) {
                            Ok(Message::Data(data)) => {
                                cancel.check()?;
                                writer.write_all(&data)?;
                                cancel.check()?;
                            }
                            Ok(Message::Flush(ack)) => {
                                writer.flush()?;
                                let _ = ack.send(());
                            }
                            Err(mpsc::RecvTimeoutError::Timeout) => continue,
                            Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        }
                    }
                    if !writer_abort.load(Ordering::Acquire) {
                        cancel.check()?;
                        writer.flush()?;
                        cancel.check()?;
                    }
                    Ok(())
                }))
                .unwrap_or_else(|_| Err(io::Error::other("Transfer writer panicked")));
            (writer, result)
        });
        let mut pipeline = PipelineWriter {
            sender,
            cancel,
            abort,
            finished: false,
        };
        let produced = cancel
            .check()
            .map_err(anyhow::Error::from)
            .and_then(|()| produce(&mut pipeline));
        pipeline.finished = produced.is_ok();
        drop(pipeline);
        // The worker catches I/O panics and returns its writer for cleanup.
        let (writer, written) = worker.join().expect("writer panic is caught");
        let result = match (produced, written) {
            (result, Ok(())) => result,
            (Ok(_), Err(error)) => Err(error).context("Background transfer writer failed"),
            (Err(producer), Err(writer)) => {
                let disconnected = error_contains::<PipelineStopped>(producer.as_ref());
                let producer_cancelled = error_contains::<DiscCancelled>(producer.as_ref());
                let writer_cancelled = error_contains::<DiscCancelled>(&writer);
                if disconnected || (producer_cancelled && !writer_cancelled) {
                    Err(writer).context("Background transfer writer failed")
                } else {
                    // Keep an actual source/conversion failure even if stopping
                    // its writer also encountered the cancellation request.
                    Err(producer)
                }
            }
        };
        (writer, result)
    })
}

/// Sequentially concatenate an image's physical parts, including ZIP temporaries
/// positioned at EOF. The caller chooses direct or queued writes independently.
pub fn copy_files<C: DiscCancellation>(
    files: &mut [File],
    output: &mut (impl Write + ?Sized),
    cancel: &C,
    mut report: impl FnMut(u64) -> Result<()>,
) -> Result<()> {
    let total = files
        .iter()
        .try_fold(0u64, |total, file| -> io::Result<u64> {
            cancel.check()?;
            total
                .checked_add(file.metadata()?.len())
                .ok_or_else(|| io::Error::other("Input size overflow"))
        })?;
    let mut written = 0u64;
    let mut buffer = vec![0; BLOCK_SIZE];
    for file in files {
        cancel.check()?;
        file.rewind()?;
        loop {
            report(written.saturating_mul(100).checked_div(total).unwrap_or(0))?;
            cancel.check()?;
            let count = file.read(&mut buffer)?;
            cancel.check()?;
            if count == 0 {
                break;
            }
            output.write_all(&buffer[..count])?;
            written += count as u64;
        }
    }
    if written != total {
        anyhow::bail!("Source image changed size while copying");
    }
    report(100)
}

pub fn copy_stream<C: DiscCancellation>(
    source: &mut impl nod::read::DiscStream,
    output: &mut dyn Write,
    cancel: &C,
    mut report: impl FnMut(u64) -> Result<()>,
) -> Result<()> {
    cancel.check()?;
    let total = source.stream_len()?;
    let mut offset = 0;
    let mut buffer = vec![0; BLOCK_SIZE];
    while offset < total {
        cancel.check()?;
        report(offset.saturating_mul(100).checked_div(total).unwrap_or(0))?;
        let count = (total - offset).min(BLOCK_SIZE as u64) as usize;
        source.read_exact_at(&mut buffer[..count], offset)?;
        cancel.check()?;
        output.write_all(&buffer[..count])?;
        offset += count as u64;
    }
    cancel.check()?;
    report(100)
}

#[cfg(test)]
mod tests;
