// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::*;
use smol::stream::StreamExt;
use std::{sync::mpsc, time::Duration};

fn synthetic_disc(seed: usize) -> Vec<u8> {
    let mut data = vec![0; 0x50000];
    data[..6].copy_from_slice(b"GZZE01");
    data[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
    for (offset, byte) in data[0x8000..].iter_mut().enumerate() {
        *byte = ((offset * (seed + 1) + seed * 47) % 251) as u8;
    }
    data
}

fn game(base: &Path, number: usize) -> Game {
    let path = base.join(format!("Cancel {number} [GZZE01]"));
    std::fs::create_dir(&path).unwrap();
    std::fs::write(path.join("game.iso"), synthetic_disc(number)).unwrap();
    smol::block_on(Game::try_from_path(path, false)).unwrap()
}

fn options(format: ExportFormat, destination: DestinationMode) -> DiscExportOptions {
    DiscExportOptions {
        format: format.format(),
        threads: ExportThreads {
            preloader: 0,
            processor: 0,
        },
        destination,
    }
}

fn directory_count(path: &Path) -> usize {
    std::fs::read_dir(path).unwrap().count()
}

fn assert_cancelled(states: &[LongOperationState]) {
    assert!(
        matches!(states.last(), Some(LongOperationState::Cancelled(_))),
        "unexpected terminal state: {states:?}"
    );
    assert_eq!(
        states
            .iter()
            .filter(|state| matches!(
                state,
                LongOperationState::Finished(_)
                    | LongOperationState::Errored(_)
                    | LongOperationState::Cancelled(_)
            ))
            .count(),
        1
    );
}

// A deadlock regression must fail the test instead of hanging the entire suite.
// The worker owns all its temporary files, including on the timeout path.
fn with_deadline(test: impl FnOnce() + Send + 'static) {
    let (done_tx, done_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
        let _ = done_tx.send(result);
    });
    let result = done_rx
        .recv_timeout(Duration::from_secs(30))
        .expect("cancelled export did not stop within 30 seconds");
    worker.join().unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

#[test]
fn pre_cancelled_single_and_batch_exports_create_no_files() {
    with_deadline(|| {
        let temp = tempfile::tempdir().unwrap();
        let games: Vec<_> = (0..4).map(|index| game(temp.path(), index)).collect();
        let destination = temp.path().join("exports");
        std::fs::create_dir(&destination).unwrap();
        let cancel = ExportCancellation::default();
        cancel.cancel();

        let single = smol::block_on(
            export_game(
                games[0].clone(),
                destination.join("single.rvz"),
                cancel.clone(),
            )
            .collect::<Vec<_>>(),
        );
        assert_cancelled(&single);
        for jobs in [1, 2, 4] {
            let batch = smol::block_on(
                export_games(
                    games.clone(),
                    destination.clone(),
                    ExportFormat::Rvz,
                    jobs,
                    cancel.clone(),
                )
                .collect::<Vec<_>>(),
            );
            assert_cancelled(&batch);
            assert_eq!(directory_count(&destination), 0);
        }
        for (index, game) in games.iter().enumerate() {
            assert_eq!(
                std::fs::read(game.path().join("game.iso")).unwrap(),
                synthetic_disc(index)
            );
        }
    });
}

#[test]
fn single_export_cancelled_during_data_preserves_existing_destination() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.iso");
    let destination = temp.path().join("existing.iso");
    let mut original = synthetic_disc(0);
    original.resize(0x500000, 0x67);
    std::fs::write(&source, &original).unwrap();
    std::fs::write(&destination, b"previous complete image").unwrap();
    let cancel = ExportCancellation::default();
    let mut cancelled_during_data = false;
    let result = export_disc(
        &source,
        &destination,
        options(ExportFormat::Iso, DestinationMode::ReplaceConfirmed),
        &cancel,
        |percentage| {
            if percentage > 0 && percentage < 100 {
                cancelled_during_data = true;
                cancel.cancel();
            }
            Ok(())
        },
    );
    assert!(cancelled_during_data);
    assert!(result.is_err());
    assert_eq!(
        std::fs::read(&destination).unwrap(),
        b"previous complete image"
    );
    assert_eq!(std::fs::read(source).unwrap(), original);
    assert_eq!(directory_count(temp.path()), 2);
}

#[test]
fn cancellation_after_flushing_staged_data_never_publishes_it() {
    for mode in [
        DestinationMode::KeepExisting,
        DestinationMode::ReplaceConfirmed,
    ] {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("output.iso");
        let replacing = matches!(mode, DestinationMode::ReplaceConfirmed);
        if replacing {
            std::fs::write(&destination, b"previous image").unwrap();
        }
        let cancel = ExportCancellation::default();
        let result = write_staged(&destination, &cancel, mode, |output| {
            output.write_all(&vec![0x52; 0x20000])?;
            output.flush()?;
            cancel.cancel();
            Ok(())
        });
        assert!(result.is_err());
        if replacing {
            assert_eq!(std::fs::read(destination).unwrap(), b"previous image");
            assert_eq!(directory_count(temp.path()), 1);
        } else {
            assert!(!destination.exists());
            assert_eq!(directory_count(temp.path()), 0);
        }
    }
}

#[test]
fn cancellation_at_final_progress_discards_every_output_format() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.iso");
    let original = synthetic_disc(0);
    std::fs::write(&source, &original).unwrap();
    for format in ExportFormat::ALL {
        let destination = temp.path().join(format!("output.{}", format.extension()));
        let export_options = options(format, DestinationMode::KeepExisting);
        // Record the exact callback sequence so cancellation happens at the
        // explicit final report after process and header rewriting, rather than
        // an earlier data callback which also happens to report 100 percent.
        let mut progress = Vec::new();
        export_disc(
            &source,
            &destination,
            export_options,
            &ExportCancellation::default(),
            |percentage| {
                progress.push(percentage);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(progress.last(), Some(&100));
        std::fs::remove_file(&destination).unwrap();

        let cancel = ExportCancellation::default();
        let mut calls = 0;
        let result = export_disc(
            &source,
            &destination,
            export_options,
            &cancel,
            |percentage| {
                assert_eq!(percentage, progress[calls]);
                calls += 1;
                if calls == progress.len() {
                    cancel.cancel();
                }
                Ok(())
            },
        );
        assert!(result.is_err(), "{format} was published after cancellation");
        assert_eq!(calls, progress.len(), "{format}");
        assert!(!destination.exists(), "{format}");
        assert_eq!(directory_count(temp.path()), 1, "{format}");
        assert_eq!(std::fs::read(&source).unwrap(), original, "{format}");
    }
}

#[test]
fn cancellation_between_sequential_jobs_keeps_completed_image() {
    let temp = tempfile::tempdir().unwrap();
    let games: Vec<_> = (0..4).map(|index| game(temp.path(), index)).collect();
    let destination = temp.path().join("exports");
    std::fs::create_dir(&destination).unwrap();
    let cancel = ExportCancellation::default();
    let started = AtomicUsize::new(0);
    let results = run_jobs(games.len(), 1, |index| {
        if cancel.is_cancelled() {
            return None;
        }
        started.fetch_add(1, Ordering::Relaxed);
        let result = export_disc(
            &games[index].path().join("game.iso"),
            &destination.join(output_filename(&games[index], None, ExportFormat::Iso)),
            options(ExportFormat::Iso, DestinationMode::KeepExisting),
            &cancel,
            |_| Ok(()),
        );
        // Synchronously cancel at the boundary after the first atomic publish.
        // A progress-channel observer would introduce a scheduling race here.
        assert!(result.is_ok());
        cancel.cancel();
        Some(result)
    })
    .unwrap();
    assert_eq!(started.load(Ordering::Relaxed), 1);
    assert!(matches!(results.first(), Some(Some(Ok(())))));
    assert!(results[1..].iter().all(Option::is_none));
    assert_eq!(directory_count(&destination), 1);
    assert_eq!(
        std::fs::read(destination.join(output_filename(&games[0], None, ExportFormat::Iso)))
            .unwrap(),
        synthetic_disc(0)
    );
    for (index, game) in games.iter().enumerate() {
        assert_eq!(
            std::fs::read(game.path().join("game.iso")).unwrap(),
            synthetic_disc(index)
        );
    }
}

#[test]
fn cancellation_stops_all_active_parallel_exports_and_cleans_temporary_files() {
    with_deadline(|| {
        let temp = tempfile::tempdir().unwrap();
        let games: Vec<_> = (0..8).map(|index| game(temp.path(), index)).collect();
        let destination = temp.path().join("exports");
        std::fs::create_dir(&destination).unwrap();
        let cancel = ExportCancellation::default();
        let (started_tx, started_rx) = mpsc::channel();
        let releases: Vec<_> = (0..4).map(|_| mpsc::channel()).collect();
        let (release_txs, release_rxs): (Vec<_>, Vec<_>) = releases
            .into_iter()
            .map(|(tx, rx)| (tx, Mutex::new(rx)))
            .unzip();
        std::thread::scope(|scope| {
            let worker = scope.spawn(|| {
                run_jobs(games.len(), 4, |index| {
                    if cancel.is_cancelled() {
                        return None;
                    }
                    let mut reported = false;
                    let result = export_disc(
                        &games[index].path().join("game.iso"),
                        &destination.join(output_filename(&games[index], None, ExportFormat::Rvz)),
                        options(ExportFormat::Rvz, DestinationMode::KeepExisting),
                        &cancel,
                        |percentage| {
                            if percentage > 0 && !reported {
                                reported = true;
                                started_tx.send(index)?;
                                release_rxs[index]
                                    .lock()
                                    .unwrap()
                                    .recv_timeout(Duration::from_secs(10))?;
                            }
                            Ok(())
                        },
                    );
                    Some(result)
                })
                .unwrap()
            });
            let mut active = Vec::new();
            for _ in 0..4 {
                active.push(started_rx.recv_timeout(Duration::from_secs(10)).unwrap());
            }
            active.sort_unstable();
            assert_eq!(active, vec![0, 1, 2, 3]);
            assert_eq!(directory_count(&destination), 4);
            assert!(std::fs::read_dir(&destination).unwrap().all(|entry| {
                entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".twbm-export-")
            }));
            cancel.cancel();
            for tx in release_txs {
                tx.send(()).unwrap();
            }
            let results = worker.join().unwrap();
            assert!(
                results[..4]
                    .iter()
                    .all(|result| matches!(result, Some(Err(_))))
            );
            assert!(results[4..].iter().all(Option::is_none));
        });
        assert_eq!(directory_count(&destination), 0);
        for (index, game) in games.iter().enumerate() {
            assert_eq!(
                std::fs::read(game.path().join("game.iso")).unwrap(),
                synthetic_disc(index)
            );
        }
    });
}

#[test]
fn cancellation_with_multiple_processor_and_preloader_threads_does_not_deadlock() {
    with_deadline(|| {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.iso");
        let destination = temp.path().join("output.rvz");
        let mut original = synthetic_disc(0);
        original.resize(0x400000, 0x67);
        std::fs::write(&source, &original).unwrap();
        let cancel = ExportCancellation::default();
        let mut cancelled_during_data = false;
        let result = export_disc(
            &source,
            &destination,
            DiscExportOptions {
                format: Format::Rvz,
                threads: ExportThreads {
                    preloader: 2,
                    processor: 2,
                },
                destination: DestinationMode::KeepExisting,
            },
            &cancel,
            |percentage| {
                if percentage > 0 {
                    cancelled_during_data = true;
                    cancel.cancel();
                    cancel.check()?;
                }
                Ok(())
            },
        );
        assert!(cancelled_during_data);
        assert!(result.is_err());
        assert!(!destination.exists());
        assert_eq!(directory_count(temp.path()), 1);
        assert_eq!(std::fs::read(source).unwrap(), original);
    });
}

#[test]
fn dropping_observer_during_export_cancels_before_publication() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.iso");
    let destination = temp.path().join("output.iso");
    let original = synthetic_disc(0);
    std::fs::write(&source, &original).unwrap();
    let (tx, rx) = smol::channel::bounded(1);
    let cancel = ExportCancellation::default().observed_by(&tx);
    let mut observer = Some(rx);
    let result = export_disc(
        &source,
        &destination,
        options(ExportFormat::Iso, DestinationMode::KeepExisting),
        &cancel,
        |percentage| {
            if percentage > 0 {
                drop(observer.take());
            }
            Ok(())
        },
    );
    assert!(cancel.is_cancelled());
    assert!(result.is_err());
    assert!(!destination.exists());
    assert_eq!(directory_count(temp.path()), 1);
    assert_eq!(std::fs::read(source).unwrap(), original);
}
