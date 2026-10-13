// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::*;
use crate::games::export::ExportCancellation;
use std::sync::Mutex;

#[derive(Default)]
struct MemoryWriter {
    bytes: Vec<u8>,
    threads: Vec<std::thread::ThreadId>,
    flushes: usize,
}
impl Write for MemoryWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.threads.push(std::thread::current().id());
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.flushes += 1;
        Ok(())
    }
}

#[test]
fn direct_and_pipelined_writes_have_identical_bytes_and_flush_before_return() {
    let expected: Vec<_> = (0..BLOCK_SIZE * 3 + 17).map(|i| (i % 251) as u8).collect();
    for fast in [false, true] {
        let caller = std::thread::current().id();
        let (writer, result) = transfer(
            MemoryWriter::default(),
            fast,
            &ExportCancellation::default(),
            |writer| {
                writer.write_all(&expected[..31])?;
                writer.flush()?;
                writer.write_all(&expected[31..])?;
                Ok(42)
            },
        );
        assert_eq!(result.unwrap(), 42);
        assert_eq!(writer.bytes, expected);
        assert!(writer.flushes >= 2);
        assert!(
            writer
                .threads
                .iter()
                .all(|thread| (*thread == caller) != fast)
        );
    }
}

struct BlockedWriter {
    entered: Option<mpsc::Sender<()>>,
    release: mpsc::Receiver<()>,
    bytes: Arc<Mutex<Vec<u8>>>,
    fail: bool,
}
impl Write for BlockedWriter {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if let Some(entered) = self.entered.take() {
            entered.send(()).unwrap();
            self.release.recv_timeout(Duration::from_secs(5)).unwrap();
        }
        if self.fail {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "real destination failure",
            ));
        }
        self.bytes.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn full_queue_cancellation_unblocks_producer_and_joins_current_write() {
    let cancel = ExportCancellation::default();
    let worker_cancel = cancel.clone();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (queued_tx, queued_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let output = bytes.clone();
    let worker = std::thread::spawn(move || {
        let writer = BlockedWriter {
            entered: Some(entered_tx),
            release: release_rx,
            bytes: output,
            fail: false,
        };
        let (_, result) = transfer(writer, true, &worker_cancel, |writer| {
            let block = vec![9; BLOCK_SIZE];
            for index in 0..100 {
                writer.write_all(&block)?;
                queued_tx.send(index + 1).unwrap();
            }
            Ok(())
        });
        done_tx.send(result).unwrap();
    });
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    for expected in 1..=QUEUE_BLOCKS + 1 {
        assert_eq!(
            queued_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            expected
        );
    }
    cancel.cancel();
    assert!(
        done_rx.try_recv().is_err(),
        "must join the still-running physical write"
    );
    release_tx.send(()).unwrap();
    let error = done_rx
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .unwrap_err();
    worker.join().unwrap();
    assert!(error_contains::<DiscCancelled>(error.as_ref()), "{error:#}");
    assert_eq!(
        bytes.lock().unwrap().len(),
        BLOCK_SIZE,
        "queued writes must not continue after cancellation"
    );
}

#[test]
fn genuine_source_error_wins_over_writer_cancellation() {
    let cancel = ExportCancellation::default();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let writer = BlockedWriter {
        entered: Some(entered_tx),
        release: release_rx,
        bytes: Arc::default(),
        fail: false,
    };
    let (_, result) = transfer(writer, true, &cancel, |writer| -> Result<()> {
        writer.write_all(&[7; 16])?;
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        cancel.cancel();
        release_tx.send(()).unwrap();
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "real source failure").into())
    });
    let error = result.unwrap_err();
    assert!(format!("{error:#}").contains("real source failure"));
    assert!(!error_contains::<DiscCancelled>(error.as_ref()));
}

#[test]
fn genuine_writer_error_wins_over_producer_cancellation_and_stopped_pipe() {
    for cancel_producer in [false, true] {
        let cancel = ExportCancellation::default();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writer = BlockedWriter {
            entered: Some(entered_tx),
            release: release_rx,
            bytes: Arc::default(),
            fail: true,
        };
        let (_, result) = transfer(writer, true, &cancel, |writer| -> Result<()> {
            writer.write_all(&[7; 16])?;
            entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            if cancel_producer {
                cancel.cancel();
            }
            release_tx.send(()).unwrap();
            if cancel_producer {
                cancel.check()?;
            }
            // Flush is acknowledged only if the writer actually succeeds.
            writer.flush()?;
            Ok(())
        });
        let error = result.unwrap_err();
        assert!(format!("{error:#}").contains("real destination failure"));
        assert!(!error_contains::<DiscCancelled>(error.as_ref()));
    }
}

#[test]
fn producer_panic_joins_and_stops_writer_before_unwinding() {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let cancel = ExportCancellation::default();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let writer = BlockedWriter {
        entered: Some(entered_tx),
        release: release_rx,
        bytes: bytes.clone(),
        fail: false,
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        transfer(writer, true, &cancel, |writer| -> Result<()> {
            writer.write_all(&[7; 16])?;
            entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            release_tx.send(()).unwrap();
            panic!("simulated producer panic");
        })
    }));
    assert!(result.is_err());
    assert_eq!(bytes.lock().unwrap().len(), 16);
}
