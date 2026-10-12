// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::{tests::*, *};
use smol::stream::StreamExt;
use std::{
    fs::File,
    io::Write,
    path::Path,
    sync::{Barrier, mpsc},
    time::Duration,
};

fn deadline(test: impl FnOnce() + Send + 'static) {
    let (tx, rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
        let _ = tx.send(result);
    });
    let result = rx
        .recv_timeout(Duration::from_secs(30))
        .expect("Cancelled import did not join its workers");
    worker.join().unwrap();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

fn large_disc(index: usize) -> Vec<u8> {
    let mut bytes = disc(index, 0);
    bytes.resize(8 * 1024 * 1024, 0x3f + index as u8);
    bytes
}

fn assert_no_staging(path: &Path) {
    if path.exists() {
        assert_eq!(std::fs::read_dir(path).unwrap().count(), 0);
    }
}

#[test]
fn cancelled_before_start_returns_every_entry_without_opening_sources() {
    for (count, jobs) in [(1, 1), (6, 1), (6, 2), (6, 4)] {
        let temp = tempfile::tempdir().unwrap();
        let entries: Vec<_> = (0..count)
            .map(|index| entry(&temp.path().join(format!("{index}.iso")), &disc(index, 0)))
            .collect();
        // These would fail if cancellation still tried to open queued files.
        for entry in &entries {
            std::fs::remove_file(entry.path()).unwrap();
        }
        let cancel = ImportCancellation::default();
        cancel.cancel();
        let events = smol::block_on(
            import_games(
                entries.clone(),
                config(temp.path()),
                DriveState::NotLoaded,
                jobs,
                cancel,
            )
            .collect::<Vec<_>>(),
        );
        assert_eq!(
            summary(&events),
            &ImportSummary {
                imported: 0,
                failed: 0,
                cancelled: count
            }
        );
        let results: Vec<_> = events
            .iter()
            .filter_map(|event| {
                if let BatchImportEvent::Result(result) = event {
                    Some(result)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(results.len(), count);
        for (entry, result) in entries.iter().zip(results) {
            assert_eq!(entry.path(), result.entry.path());
            assert!(result.cancelled && result.error.is_none() && result.warning.is_none());
        }
        assert_no_staging(temp.path());
    }
}

#[test]
fn copying_and_nod_conversion_cancel_cleanly_with_threaded_readers() {
    deadline(|| {
        for compressed in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let bytes = large_disc(0);
            let input = if compressed {
                rvz(bytes.clone())
            } else {
                bytes.clone()
            };
            let source = entry(
                &temp.path().join(if compressed {
                    "source.rvz"
                } else {
                    "source.iso"
                }),
                &input,
            );
            let mut config = config(temp.path());
            drop(config.update(crate::config::message::ConfigMessage::SetRemoveSourcesGames(true)));
            let plan = ImportPlan::new(&source, &config, &DriveState::NotLoaded);
            let cancel = ImportCancellation::default();
            let result = perform_blocking(
                &source,
                &config,
                &plan,
                ImportThreads {
                    preloader: 2,
                    processor: 2,
                },
                &Mutex::new(()),
                &cancel,
                |percentage| {
                    if percentage > 0 {
                        cancel.cancel();
                    }
                    Ok(())
                },
            );
            let outcome = import_result(&source, result);
            assert!(
                outcome.cancelled,
                "Expected typed cancellation: {outcome:?}"
            );
            assert!(outcome.error.is_none(), "{outcome:?}");
            assert!(outcome.warning.is_none(), "{outcome:?}");
            assert_no_staging(&temp.path().join("games"));
            assert_eq!(std::fs::read(source.path()).unwrap(), input);
        }
    });
}

#[test]
fn sequential_cancellation_keeps_committed_game_and_skips_remaining_queue() {
    let temp = tempfile::tempdir().unwrap();
    let config = config(temp.path());
    let entries: Vec<_> = (0..5)
        .map(|index| {
            entry(
                &temp.path().join(format!("{index}.iso")),
                &large_disc(index),
            )
        })
        .collect();
    let cancel = ImportCancellation::default();
    let commit = Mutex::new(());
    let results = run_jobs(
        entries.len(),
        1,
        || cancel.is_cancelled(),
        |index| {
            let entry = &entries[index];
            let plan = ImportPlan::new(entry, &config, &DriveState::NotLoaded);
            import_result(
                entry,
                perform_blocking(
                    entry,
                    &config,
                    &plan,
                    ImportThreads::for_jobs(2),
                    &commit,
                    &cancel,
                    |percentage| {
                        if index == 1 && percentage > 0 {
                            cancel.cancel();
                        }
                        Ok(())
                    },
                ),
            )
        },
    );
    assert_eq!(results.len(), 2);
    assert!(!results[0].cancelled && results[0].error.is_none());
    assert!(results[1].cancelled && results[1].error.is_none());
    assert_eq!(
        std::fs::read(destination(&entries[0], &config)).unwrap(),
        large_disc(0)
    );
    for entry in &entries[1..] {
        assert!(!destination(entry, &config).exists());
    }
    assert_eq!(
        std::fs::read_dir(temp.path().join("games"))
            .unwrap()
            .count(),
        1
    );
    for entry in &entries {
        assert!(entry.path().exists());
    }
}

#[test]
fn parallel_cancellation_waits_for_all_active_writers_and_cleans_every_part() {
    deadline(|| {
        for jobs in [2, 4] {
            let temp = tempfile::tempdir().unwrap();
            let config = config(temp.path());
            let entries: Vec<_> = (0..8)
                .map(|index| {
                    entry(
                        &temp.path().join(format!("{index}.rvz")),
                        &rvz(large_disc(index)),
                    )
                })
                .collect();
            let cancel = ImportCancellation::default();
            let barrier = Barrier::new(jobs);
            let commit = Mutex::new(());
            let results = run_jobs(
                entries.len(),
                jobs,
                || cancel.is_cancelled(),
                |index| {
                    let entry = &entries[index];
                    let plan = ImportPlan::new(entry, &config, &DriveState::NotLoaded);
                    let mut ready = false;
                    let result = perform_blocking(
                        entry,
                        &config,
                        &plan,
                        ImportThreads::for_jobs(jobs),
                        &commit,
                        &cancel,
                        |percentage| {
                            if percentage > 0 && !ready {
                                ready = true;
                                barrier.wait();
                                cancel.cancel();
                            }
                            Ok(())
                        },
                    );
                    import_result(entry, result)
                },
            );
            assert_eq!(results.len(), jobs);
            assert!(
                results
                    .iter()
                    .all(|result| result.cancelled && result.error.is_none()),
                "{results:?}"
            );
            assert_no_staging(&temp.path().join("games"));
            for entry in &entries {
                assert!(entry.path().exists());
            }
        }
    });
}

#[test]
fn final_header_and_sync_cancellation_remove_all_staged_parts() {
    for cancel_at in [4, 7] {
        let temp = tempfile::tempdir().unwrap();
        let cancel = ImportCancellation::default();
        let mut writer = StagedWriter::new(
            temp.path(),
            |part| format!("part{part}"),
            NonZeroUsize::new(4),
            cancel.clone(),
        )
        .unwrap();
        writer.write_all(b"abcdefghijklmnopq").unwrap();
        let mut checks = 0;
        let result = writer.finish_with_check(b"123456789", || {
            checks += 1;
            if checks == cancel_at {
                cancel.cancel();
            }
            Ok(())
        });
        let error = result.err().expect("Finalization should be cancelled");
        assert!(is_cancel_error(&error), "{error:#}");
        assert_no_staging(temp.path());
    }
}

#[test]
fn publication_cancellation_rolls_back_only_its_own_parts() {
    for existing_folder in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("game");
        if existing_folder {
            std::fs::create_dir(&target).unwrap();
            std::fs::write(target.join("disc2.iso"), b"completed second disc").unwrap();
        }
        let cancel = ImportCancellation::default();
        let mut writer = StagedWriter::new(
            temp.path(),
            |part| format!("part{part}"),
            NonZeroUsize::new(4),
            cancel.clone(),
        )
        .unwrap();
        writer.write_all(b"abcdefghijklmnopq").unwrap();
        let staged = writer.finish(&[]).unwrap();
        let result = staged.publish_with_check(&target, |published| {
            if published == 1 {
                assert!(target.join("part1").exists());
                cancel.cancel();
            }
            cancel.check()?;
            Ok(())
        });
        assert!(is_cancel_error(&result.unwrap_err()));
        if existing_folder {
            assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
            assert_eq!(
                std::fs::read(target.join("disc2.iso")).unwrap(),
                b"completed second disc"
            );
        } else {
            assert!(!target.exists());
        }
        assert_eq!(
            std::fs::read_dir(temp.path()).unwrap().count(),
            usize::from(existing_folder)
        );
    }
}

#[test]
fn cancellation_racing_with_last_atomic_rename_keeps_complete_image() {
    let temp = tempfile::tempdir().unwrap();
    let cancel = ImportCancellation::default();
    let mut writer = StagedWriter::new(
        temp.path(),
        |part| format!("part{part}"),
        NonZeroUsize::new(4),
        cancel.clone(),
    )
    .unwrap();
    writer.write_all(b"abcdefghijkl").unwrap();
    let target = temp.path().join("output");
    let result = writer
        .finish(&[])
        .unwrap()
        .publish_with_check(&target, |published| {
            cancel.check()?;
            // Model cancellation arriving after the final pre-rename check.
            if published == 2 {
                cancel.cancel();
            }
            Ok(())
        });
    assert!(result.is_ok());
    assert!(cancel.is_cancelled());
    assert_eq!(std::fs::read(target.join("part0")).unwrap(), b"abcd");
    assert_eq!(std::fs::read(target.join("part1")).unwrap(), b"efgh");
    assert_eq!(std::fs::read(target.join("part2")).unwrap(), b"ijkl");
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
}

#[test]
fn zip_extraction_and_cloned_readers_observe_import_cancellation() {
    #[derive(Clone)]
    struct Counted {
        cancel: ImportCancellation,
        checks: Arc<AtomicUsize>,
    }
    impl DiscCancellation for Counted {
        fn check(&self) -> io::Result<()> {
            if self.checks.fetch_add(1, Ordering::SeqCst) == 9 {
                self.cancel.cancel();
            }
            self.cancel.check()
        }
    }
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("source.zip");
    let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
    zip.start_file("game.iso", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(&large_disc(0)).unwrap();
    zip.finish().unwrap();
    let original = std::fs::read(&path).unwrap();
    let cancel = Counted {
        cancel: ImportCancellation::default(),
        checks: Arc::new(AtomicUsize::new(0)),
    };
    let error = retrieve_multi_disc_files_cancellable(&path, &cancel).unwrap_err();
    assert!(is_cancel_error(&error));
    assert_eq!(std::fs::read(&path).unwrap(), original);

    use nod::read::DiscStream;
    let source = entry(&temp.path().join("game.iso"), &disc(0, 0));
    let (tx, rx) = smol::channel::bounded(1);
    let cancel = ImportCancellation::default().observed_by(&tx);
    let files = retrieve_multi_disc_files_cancellable(source.path(), &cancel).unwrap();
    let reader = CancellableDiscStream::new(SharedMultiFileReader::new(files).unwrap(), cancel);
    let mut clone = reader.clone();
    drop(rx);
    let error = anyhow::Error::new(clone.read_exact_at(&mut [0; 6], 0).unwrap_err());
    assert!(is_cancel_error(&error));
}

#[test]
fn cancellation_does_not_hide_unrelated_io_errors_or_cleanup_failures() {
    let temp = tempfile::tempdir().unwrap();
    let source = entry(&temp.path().join("source.iso"), &disc(0, 0));
    let cancel = ImportCancellation::default();
    cancel.cancel();
    let actual_error = import_result(
        &source,
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "disk is read-only").into()),
    );
    assert!(!actual_error.cancelled && actual_error.error.is_some());
    let wrapped = anyhow::Error::new(cancel.check().unwrap_err())
        .context(CleanupWarning("cleanup failed".into()));
    let outcome = import_result(&source, Err(wrapped));
    assert!(outcome.cancelled && outcome.error.is_none());
    assert!(outcome.warning.unwrap().contains("cleanup failed"));
}

#[test]
fn cancellation_marker_survives_callback_nod_and_nested_io_boundaries() {
    let wrap = |error| {
        let callback = callback_io_error(error);
        let nod = nod::Error::Io("Failed to write disc data".into(), callback);
        anyhow::Error::new(io::Error::other(io::Error::other(nod))).context("Finalizing image")
    };
    let cancel = ImportCancellation::default();
    cancel.cancel();
    let cancelled =
        wrap(anyhow::Error::new(cancel.check().unwrap_err()).context("Progress callback"));
    assert!(is_cancel_error(&cancelled), "{cancelled:#}");
    let genuine = wrap(io::Error::new(io::ErrorKind::PermissionDenied, "read-only disk").into());
    assert!(!is_cancel_error(&genuine));
}
