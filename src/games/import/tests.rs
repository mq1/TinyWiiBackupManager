// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use super::*;
use crate::config::message::ConfigMessage;
use smol::stream::StreamExt;
use std::{
    fs::File,
    io::{Cursor, Read, Seek, Write},
    path::Path,
    sync::{Arc, Barrier, atomic::AtomicBool},
};

pub(super) fn disc(index: usize, number: u8) -> Vec<u8> {
    let mut bytes = vec![0; 0x50000];
    bytes[..6].copy_from_slice(b"GZZE01");
    bytes[1] = b'A' + index as u8;
    bytes[6] = number;
    bytes[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
    bytes[0x20..0x2b].copy_from_slice(b"Import Test");
    for (offset, byte) in bytes[0x8000..].iter_mut().enumerate() {
        *byte = ((offset * (index + 1) + usize::from(number) * 13 + index * 47) % 251) as u8;
    }
    bytes
}

fn wii_disc() -> Vec<u8> {
    // A real partition layout, including one complete Wii hash group. A Wii
    // magic placed in a GC fixture is not sufficient for RVZ reconstruction.
    const PARTITION: usize = 0x50000;
    const DATA: usize = 0x70000;
    const GROUP: usize = 64 * 0x8000;
    let mut bytes = vec![0; DATA + GROUP];
    bytes[..6].copy_from_slice(b"SYNX01");
    bytes[0x18..0x1c].copy_from_slice(&[0x5d, 0x1c, 0x9e, 0xa3]);
    bytes[0x20..0x2b].copy_from_slice(b"Import Test");
    bytes[0x61] = 1; // Decrypted sectors retain their hash areas.
    let put = |bytes: &mut [u8], offset, value: u32| {
        bytes[offset..offset + 4].copy_from_slice(&value.to_be_bytes())
    };
    put(&mut bytes, 0x40000, 1);
    put(&mut bytes, 0x40004, 0x40020 / 4);
    put(&mut bytes, 0x40020, PARTITION as u32 / 4);
    let issuer = b"Root-CA00000001-XS00000003";
    bytes[PARTITION + 0x140..PARTITION + 0x140 + issuer.len()].copy_from_slice(issuer);
    put(&mut bytes, PARTITION + 0x2b4, 0x8000 / 4);
    put(&mut bytes, PARTITION + 0x2b8, 0x20000 / 4);
    put(&mut bytes, PARTITION + 0x2bc, GROUP as u32 / 4);
    bytes[PARTITION + 0x8000..DATA].fill(0xa7);
    for (offset, byte) in bytes[DATA..].iter_mut().enumerate() {
        *byte = ((offset * 7) >> 8) as u8;
    }
    let header = bytes[..0x400].to_vec();
    let boot = DATA + 0x400;
    bytes[boot..boot + 0x440].fill(0);
    bytes[boot..boot + header.len()].copy_from_slice(&header);
    put(&mut bytes, boot + 0x424, 0x1000 / 4);
    put(&mut bytes, boot + 0x428, 16 / 4);
    put(&mut bytes, boot + 0x42c, 16 / 4);
    bytes[boot + 0x1000..boot + 0x1010].fill(0);
    bytes[boot + 0x1000] = 1;
    put(&mut bytes, boot + 0x1008, 1);
    bytes
}

pub(super) fn config(path: &Path) -> Config {
    let mut config = Config::default();
    drop(config.set_mount_point(path.to_path_buf()));
    config
}

pub(super) fn entry(path: &Path, bytes: &[u8]) -> ImportEntry {
    std::fs::write(path, bytes).unwrap();
    smol::block_on(ImportEntry::new(path.to_path_buf())).unwrap()
}

pub(super) fn destination(entry: &ImportEntry, config: &Config) -> PathBuf {
    let plan = ImportPlan::new(entry, config, &DriveState::NotLoaded);
    plan.folder.join(plan.filename(entry, config, 0))
}

pub(super) fn summary(events: &[BatchImportEvent]) -> &ImportSummary {
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, BatchImportEvent::Finished(_)))
            .count(),
        1
    );
    match events.last().unwrap() {
        BatchImportEvent::Finished(summary) => summary,
        other => panic!("Missing final summary: {other:?}"),
    }
}

pub(super) fn rvz(bytes: Vec<u8>) -> Vec<u8> {
    let reader =
        DiscReader::new_from_cloneable_read(Cursor::new(bytes), &DiscOptions::default()).unwrap();
    let writer = DiscWriter::new(reader, &FormatOptions::new(Format::Rvz)).unwrap();
    let mut output = Cursor::new(Vec::new());
    let finalization = writer
        .process(
            |bytes, _, _| output.write_all(&bytes),
            &ProcessOptions::default(),
        )
        .unwrap();
    output.rewind().unwrap();
    output.write_all(&finalization.header).unwrap();
    let mut bytes = output.into_inner();
    // The metadata probe reads a full 32 KiB even for tiny synthetic archives.
    bytes.resize(bytes.len().max(0x8000), 0);
    bytes
}

#[test]
fn scheduler_reaches_and_respects_requested_concurrency() {
    for jobs in [1, 2, 4] {
        let barrier = Barrier::new(jobs);
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let results = run_jobs(
            jobs * 3,
            jobs,
            || false,
            |index| {
                let running = active.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(running, Ordering::SeqCst);
                barrier.wait();
                active.fetch_sub(1, Ordering::SeqCst);
                barrier.wait();
                index
            },
        );
        assert_eq!(results, (0..jobs * 3).collect::<Vec<_>>());
        assert_eq!(peak.load(Ordering::SeqCst), jobs);
    }
}

#[test]
fn scheduler_stops_claiming_work_when_observer_closes() {
    let stopped = AtomicBool::new(false);
    let results = run_jobs(
        12,
        1,
        || stopped.load(Ordering::Acquire),
        |index| {
            stopped.store(true, Ordering::Release);
            index
        },
    );
    assert_eq!(results, [0]);
}

#[test]
fn sequential_and_parallel_imports_preserve_distinct_disc_contents() {
    for (jobs, fast) in [
        (1, false),
        (2, false),
        (4, false),
        (1, true),
        (2, true),
        (4, true),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("usb");
        std::fs::create_dir(&target).unwrap();
        let mut config = config(&target);
        drop(config.update(ConfigMessage::SetFastTransfer(fast)));
        let entries: Vec<_> = (0..6)
            .map(|index| {
                let bytes = disc(index, 0);
                let compressed = index % 2 == 0;
                let source = temp.path().join(format!(
                    "source{index}.{}",
                    if compressed { "rvz" } else { "iso" }
                ));
                entry(&source, &if compressed { rvz(bytes) } else { bytes })
            })
            .collect();
        let events = smol::block_on(
            import_games(
                entries.clone(),
                config.clone(),
                DriveState::NotLoaded,
                jobs,
                ImportCancellation::default(),
            )
            .collect::<Vec<_>>(),
        );
        assert_eq!(
            summary(&events),
            &ImportSummary {
                imported: 6,
                failed: 0,
                cancelled: 0
            },
            "{events:#?}"
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, BatchImportEvent::Result(_)))
                .count(),
            6
        );
        for (index, entry) in entries.iter().enumerate() {
            assert_eq!(
                std::fs::read(destination(entry, &config)).unwrap(),
                disc(index, 0)
            );
            assert!(entry.path().exists());
        }
        assert!(
            std::fs::read_dir(target.join("games"))
                .unwrap()
                .all(|entry| !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".twbm-"))
        );
    }
}

#[test]
fn parallel_gc_discs_share_folder_without_overwriting_each_other() {
    let temp = tempfile::tempdir().unwrap();
    let config = config(temp.path());
    let entries: Vec<_> = (0..2)
        .map(|number| {
            entry(
                &temp.path().join(format!("disc{number}.iso")),
                &disc(0, number),
            )
        })
        .collect();
    let events = smol::block_on(
        import_games(
            entries.clone(),
            config.clone(),
            DriveState::NotLoaded,
            2,
            ImportCancellation::default(),
        )
        .collect::<Vec<_>>(),
    );
    assert_eq!(
        summary(&events),
        &ImportSummary {
            imported: 2,
            failed: 0,
            cancelled: 0
        },
        "{events:#?}"
    );
    assert_eq!(
        destination(&entries[0], &config).parent(),
        destination(&entries[1], &config).parent()
    );
    for (number, entry) in entries.iter().enumerate() {
        assert_eq!(
            std::fs::read(destination(entry, &config)).unwrap(),
            disc(0, number as u8)
        );
    }
}

#[test]
fn duplicate_inputs_existing_files_and_missing_source_do_not_block_other_games() {
    let temp = tempfile::tempdir().unwrap();
    let config = config(temp.path());
    let duplicate = entry(&temp.path().join("duplicate.iso"), &disc(0, 0));
    let existing = entry(&temp.path().join("existing.iso"), &disc(1, 0));
    let missing = entry(&temp.path().join("missing.iso"), &disc(2, 0));
    let good = entry(&temp.path().join("good.iso"), &disc(3, 0));
    let existing_output = destination(&existing, &config);
    std::fs::create_dir_all(existing_output.parent().unwrap()).unwrap();
    std::fs::write(&existing_output, b"previous complete image").unwrap();
    std::fs::remove_file(missing.path()).unwrap();
    let events = smol::block_on(
        import_games(
            vec![
                duplicate.clone(),
                duplicate.clone(),
                existing,
                missing,
                good.clone(),
            ],
            config.clone(),
            DriveState::NotLoaded,
            4,
            ImportCancellation::default(),
        )
        .collect::<Vec<_>>(),
    );
    assert_eq!(
        summary(&events),
        &ImportSummary {
            imported: 1,
            failed: 4,
            cancelled: 0
        },
        "{events:#?}"
    );
    assert_eq!(
        std::fs::read(existing_output).unwrap(),
        b"previous complete image"
    );
    assert!(!destination(&duplicate, &config).exists());
    assert_eq!(
        std::fs::read(destination(&good, &config)).unwrap(),
        disc(3, 0)
    );
}

#[test]
fn zip_same_format_is_rewound_and_copied_completely() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.zip");
    let bytes = disc(0, 0);
    let mut zip = zip::ZipWriter::new(File::create(&source).unwrap());
    zip.start_file("game.iso", zip::write::SimpleFileOptions::default())
        .unwrap();
    zip.write_all(&bytes).unwrap();
    zip.finish().unwrap();
    let entry = smol::block_on(ImportEntry::new(source)).unwrap();
    let config = config(temp.path());
    let events = smol::block_on(
        import_games(
            vec![entry.clone()],
            config.clone(),
            DriveState::NotLoaded,
            1,
            ImportCancellation::default(),
        )
        .collect::<Vec<_>>(),
    );
    assert_eq!(
        summary(&events),
        &ImportSummary {
            imported: 1,
            failed: 0,
            cancelled: 0
        },
        "{events:#?}"
    );
    assert_eq!(std::fs::read(destination(&entry, &config)).unwrap(), bytes);
}

#[test]
fn split_header_rewrite_crosses_parts_and_commit_rolls_back_on_collision() {
    let temp = tempfile::tempdir().unwrap();
    let make = || {
        let mut writer = StagedWriter::new(
            temp.path(),
            |part| format!("part{part}"),
            NonZeroUsize::new(4),
            ImportCancellation::default(),
        )
        .unwrap();
        writer.write_all(b"abcdefghijklmnopq").unwrap();
        writer.finish(b"123456789").unwrap()
    };
    let output = temp.path().join("game");
    make()
        .publish(&output, &ImportCancellation::default())
        .unwrap();
    let combined: Vec<_> = (0..5)
        .flat_map(|part| std::fs::read(output.join(format!("part{part}"))).unwrap())
        .collect();
    assert_eq!(combined, b"123456789jklmnopq");
    let conflict = temp.path().join("conflict");
    std::fs::create_dir(&conflict).unwrap();
    std::fs::write(conflict.join("part0"), b"existing").unwrap();
    assert!(
        make()
            .publish(&conflict, &ImportCancellation::default())
            .is_err()
    );
    assert_eq!(std::fs::read(conflict.join("part0")).unwrap(), b"existing");
    assert_eq!(std::fs::read_dir(&conflict).unwrap().count(), 1);
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
}

#[test]
fn write_failure_and_failure_after_sync_remove_all_staging_files() {
    for fail_after_sync in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let config = config(temp.path());
        let entry = entry(&temp.path().join("source.iso"), &disc(0, 0));
        let plan = ImportPlan::new(&entry, &config, &DriveState::NotLoaded);
        let mut finished_callbacks = 0;
        let result = perform_blocking(
            &entry,
            &config,
            &plan,
            ImportThreads::for_jobs(2),
            &Mutex::new(()),
            &ImportCancellation::default(),
            |percentage| {
                if percentage == 100 {
                    finished_callbacks += 1;
                    // The copy loop reports EOF, then copy completion; the third
                    // 100% callback comes after staged parts are flushed/synced.
                    if finished_callbacks == if fail_after_sync { 3 } else { 1 } {
                        bail!("Simulated failure");
                    }
                }
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(!plan.folder.exists());
        assert_eq!(
            std::fs::read_dir(temp.path().join("games"))
                .unwrap()
                .count(),
            0
        );
        assert_eq!(std::fs::read(entry.path()).unwrap(), disc(0, 0));
    }
}

#[test]
fn sources_are_only_removed_after_successful_publication() {
    let temp = tempfile::tempdir().unwrap();
    let mut config = config(temp.path());
    drop(config.update(ConfigMessage::SetRemoveSourcesGames(true)));
    let success = entry(&temp.path().join("success.iso"), &disc(0, 0));
    let failure = entry(&temp.path().join("failure.iso"), &disc(1, 0));
    let existing = destination(&failure, &config);
    std::fs::create_dir_all(existing.parent().unwrap()).unwrap();
    std::fs::write(existing, b"keep").unwrap();
    let events = smol::block_on(
        import_games(
            vec![success.clone(), failure.clone()],
            config.clone(),
            DriveState::NotLoaded,
            2,
            ImportCancellation::default(),
        )
        .collect::<Vec<_>>(),
    );
    assert_eq!(
        summary(&events),
        &ImportSummary {
            imported: 1,
            failed: 1,
            cancelled: 0
        }
    );
    assert!(!success.path().exists());
    assert!(failure.path().exists());
    assert_eq!(
        std::fs::read(destination(&success, &config)).unwrap(),
        disc(0, 0)
    );
}

#[test]
fn staging_drops_open_handles_before_removing_temporary_directory_on_panic() {
    let temp = tempfile::tempdir().unwrap();
    let path = Arc::new(temp.path().to_path_buf());
    let result = std::panic::catch_unwind(move || {
        let mut writer = StagedWriter::new(
            &path,
            |part| format!("part{part}"),
            NonZeroUsize::new(4),
            ImportCancellation::default(),
        )
        .unwrap();
        writer.write_all(b"partial write").unwrap();
        panic!("simulated panic");
    });
    assert!(result.is_err());
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[test]
fn unavailable_mount_and_invalid_formats_fail_before_writing() {
    let temp = tempfile::tempdir().unwrap();
    let entry = entry(&temp.path().join("source.iso"), &disc(0, 0));
    let mut invalid_format = config(temp.path());
    drop(invalid_format.update(ConfigMessage::SetGcOutputFormat(Format::Rvz)));
    for config in [
        Config::default(),
        config(&temp.path().join("missing")),
        invalid_format,
    ] {
        let events = smol::block_on(
            import_games(
                vec![entry.clone()],
                config,
                DriveState::NotLoaded,
                2,
                ImportCancellation::default(),
            )
            .collect::<Vec<_>>(),
        );
        assert_eq!(
            summary(&events),
            &ImportSummary {
                imported: 0,
                failed: 1,
                cancelled: 0
            }
        );
    }
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
}

#[test]
fn fast_pipeline_drains_before_split_header_rewrite_and_publication() {
    let temp = tempfile::tempdir().unwrap();
    let cancel = ImportCancellation::default();
    let writer = StagedWriter::new(
        temp.path(),
        |part| format!("part{part}"),
        NonZeroUsize::new(64 * 1024),
        cancel.clone(),
    )
    .unwrap();
    let mut expected: Vec<_> = (0..3 * 1024 * 1024 + 17).map(|i| (i % 251) as u8).collect();
    let (writer, result) = transfer(writer, true, &cancel, |writer| {
        writer.write_all(&expected)?;
        Ok(())
    });
    result.unwrap();
    let header = vec![0x42; 96 * 1024];
    expected[..header.len()].copy_from_slice(&header);
    let target = temp.path().join("game");
    writer
        .finish(&header)
        .unwrap()
        .publish(&target, &cancel)
        .unwrap();
    let actual: Vec<_> = (0..expected.len().div_ceil(64 * 1024))
        .flat_map(|part| std::fs::read(target.join(format!("part{part}"))).unwrap())
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
}

#[test]
fn same_format_wbfs_copy_flushes_and_closes_before_publication_and_source_removal() {
    let reader =
        DiscReader::new_from_cloneable_read(Cursor::new(wii_disc()), &DiscOptions::default())
            .unwrap();
    let writer = DiscWriter::new(reader, &FormatOptions::new(Format::Wbfs)).unwrap();
    let mut output = Cursor::new(Vec::new());
    let finalization = writer
        .process(
            |bytes, _, _| output.write_all(&bytes),
            &ProcessOptions::default(),
        )
        .unwrap();
    output.rewind().unwrap();
    output.write_all(&finalization.header).unwrap();
    let mut expected = output.into_inner();
    // Same-format copying must preserve even a non-aligned container trailer.
    // This leaves data buffered at EOF and detects publication before flush.
    expected.extend_from_slice(b"WBFS copy trailer");

    for fast in [false, true] {
        for split_size in [None, NonZeroUsize::new(256 * 1024 + 17)] {
            let temp = tempfile::tempdir().unwrap();
            let target = temp.path().join("usb");
            std::fs::create_dir(&target).unwrap();
            let source = entry(&temp.path().join("source.wbfs"), &expected);
            let mut config = config(&target);
            for message in [
                ConfigMessage::SetWiiOutputFormat(Format::Wbfs),
                ConfigMessage::SetScrubUpdatePartition(false),
                ConfigMessage::SetFastTransfer(fast),
                ConfigMessage::SetRemoveSourcesGames(true),
            ] {
                drop(config.update(message));
            }
            let mut plan = ImportPlan::new(&source, &config, &DriveState::NotLoaded);
            // Exercise physical splitting without allocating a 4 GiB fixture.
            plan.split_size = split_size;
            assert_eq!(
                plan.format,
                wii_disc_info_to_nod_format(source.meta().format())
            );
            let mut completed_callbacks = 0;
            let warning = perform_blocking(
                &source,
                &config,
                &plan,
                ImportThreads::for_jobs(1),
                &Mutex::new(()),
                &ImportCancellation::default(),
                |percentage| {
                    assert!(source.path().exists());
                    assert!(!plan.folder.exists());
                    completed_callbacks += usize::from(percentage == 100);
                    Ok(())
                },
            )
            .unwrap();
            assert!(warning.is_none(), "{warning:?}");
            assert!(completed_callbacks > 0);
            assert!(!source.path().exists());
            let part_count = split_size.map_or(1, |size| expected.len().div_ceil(size.get()));
            let actual: Vec<_> = (0..part_count)
                .flat_map(|part| {
                    std::fs::read(plan.folder.join(plan.filename(&source, &config, part))).unwrap()
                })
                .collect();
            assert_eq!(actual, expected, "fast={fast}, split_size={split_size:?}");
            assert_eq!(std::fs::read_dir(&plan.folder).unwrap().count(), part_count);
            let published: Vec<_> = std::fs::read_dir(target.join("wbfs"))
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect();
            assert_eq!(published, [plan.folder]);
        }
    }
}

#[test]
fn fast_and_direct_import_match_for_all_supported_console_formats() {
    for (is_wii, format) in [
        (false, Format::Iso),
        (false, Format::Ciso),
        (true, Format::Iso),
        (true, Format::Wbfs),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let original = if is_wii { wii_disc() } else { disc(0, 0) };
        let source = entry(&temp.path().join("source.rvz"), &rvz(original.clone()));
        let mut outputs = Vec::new();
        for fast in [false, true] {
            let target = temp.path().join(format!("out-{fast}"));
            std::fs::create_dir(&target).unwrap();
            let mut config = config(&target);
            drop(config.update(ConfigMessage::SetFastTransfer(fast)));
            drop(config.update(if is_wii {
                ConfigMessage::SetWiiOutputFormat(format)
            } else {
                ConfigMessage::SetGcOutputFormat(format)
            }));
            let events = smol::block_on(
                import_games(
                    vec![source.clone()],
                    config.clone(),
                    DriveState::NotLoaded,
                    2,
                    ImportCancellation::default(),
                )
                .collect::<Vec<_>>(),
            );
            assert_eq!(
                summary(&events),
                &ImportSummary {
                    imported: 1,
                    failed: 0,
                    cancelled: 0
                },
                "{format}: {events:?}"
            );
            let output = destination(&source, &config);
            let bytes = std::fs::read(&output).unwrap();
            let reader = DiscReader::new(output, &DiscOptions::default()).unwrap();
            let mut decoded = Vec::new();
            reader
                .take(original.len() as u64)
                .read_to_end(&mut decoded)
                .unwrap();
            assert!(
                decoded == original,
                "Decoded {format} output changed (is_wii={is_wii}, fast={fast})"
            );
            outputs.push(bytes);
        }
        assert_eq!(outputs[0], outputs[1], "{format}");
    }
}
