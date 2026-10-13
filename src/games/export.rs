// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::{
        disc_reader::{
            CancellableDiscStream, DiscCancellation, DiscCancelled, SharedMultiFileReader,
            error_contains, retrieve_multi_disc_files_cancellable,
        },
        game::Game,
        import::make_game_dir_name,
    },
    long_operation::LongOperationState,
    util::misc::OPTIMAL_THREADS,
};
use anyhow::{Context, Result, bail};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use smol::stream::Stream;
use std::{
    collections::BTreeMap,
    ffi::OsStr,
    fs::File,
    io::{self, BufWriter, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

pub const MAX_PARALLEL_EXPORTS: usize = 4;
pub const DEFAULT_PARALLEL_EXPORTS: usize = 2;

#[cfg(test)]
mod cancel_tests;

#[derive(Debug, Clone, Default)]
pub struct ExportCancellation {
    requested: Arc<AtomicBool>,
    observer: Option<smol::channel::Sender<LongOperationState>>,
}

impl ExportCancellation {
    pub fn cancel(&self) {
        self.requested.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.requested.load(Ordering::Acquire)
            || self
                .observer
                .as_ref()
                .is_some_and(|sender| sender.is_closed())
    }

    pub(crate) fn check(&self) -> io::Result<()> {
        if self.is_cancelled() {
            // Interrupted is retried by read_exact/write_all and would loop forever.
            Err(io::Error::other(DiscCancelled("Export")))
        } else {
            Ok(())
        }
    }

    fn observed_by(mut self, sender: &smol::channel::Sender<LongOperationState>) -> Self {
        self.observer = Some(sender.clone());
        self
    }
}

impl DiscCancellation for ExportCancellation {
    fn check(&self) -> io::Result<()> {
        ExportCancellation::check(self)
    }
}

struct CancellableWriter<'a> {
    file: &'a mut File,
    cancel: &'a ExportCancellation,
}

impl Write for CancellableWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.cancel.check()?;
        let written = self.file.write(&bytes[..bytes.len().min(1024 * 1024)])?;
        self.cancel.check()?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.cancel.check()?;
        self.file.flush()?;
        self.cancel.check()
    }
}

impl Seek for CancellableWriter<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.cancel.check()?;
        self.file.seek(position)
    }
}

#[derive(Clone, Copy)]
enum DestinationMode {
    KeepExisting,
    ReplaceConfirmed,
}

#[derive(Clone, Copy)]
struct DiscExportOptions {
    format: Format,
    threads: ExportThreads,
    destination: DestinationMode,
}

impl DiscExportOptions {
    fn batch(format: ExportFormat, threads: ExportThreads) -> Self {
        Self {
            format: format.format(),
            threads,
            destination: DestinationMode::KeepExisting,
        }
    }
}

#[derive(Clone, Copy)]
struct ExportThreads {
    preloader: usize,
    processor: usize,
}

impl ExportThreads {
    fn for_jobs(jobs: usize) -> Self {
        if jobs == 1 {
            return Self {
                preloader: OPTIMAL_THREADS.preloader,
                processor: OPTIMAL_THREADS.processor,
            };
        }
        let cores = std::thread::available_parallelism().map_or(1, |count| count.get());
        Self {
            preloader: 0,
            // Each game already has a writer and three digest threads. Use at
            // most one extra processor to avoid nod buffering out-of-order blocks
            // from multiple compression workers for every concurrent game.
            processor: (cores / jobs).saturating_sub(4).min(1),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ExportFormat {
    #[default]
    Rvz,
    Iso,
    Wbfs,
    Ciso,
    Gcz,
    Wia,
}

impl ExportFormat {
    pub const ALL: [Self; 6] = [
        Self::Rvz,
        Self::Iso,
        Self::Wbfs,
        Self::Ciso,
        Self::Gcz,
        Self::Wia,
    ];

    pub fn extension(self) -> &'static str {
        match self {
            Self::Rvz => "rvz",
            Self::Iso => "iso",
            Self::Wbfs => "wbfs",
            Self::Ciso => "ciso",
            Self::Gcz => "gcz",
            Self::Wia => "wia",
        }
    }

    fn format(self) -> Format {
        match self {
            Self::Rvz => Format::Rvz,
            Self::Iso => Format::Iso,
            Self::Wbfs => Format::Wbfs,
            Self::Ciso => Format::Ciso,
            Self::Gcz => Format::Gcz,
            Self::Wia => Format::Wia,
        }
    }
}

impl std::fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format().fmt(f)
    }
}

pub fn ext_to_format(ext: &str) -> Option<Format> {
    match ext.to_ascii_lowercase().as_str() {
        "iso" => Some(Format::Iso),
        "ciso" => Some(Format::Ciso),
        "gcz" => Some(Format::Gcz),
        "rvz" => Some(Format::Rvz),
        "wbfs" => Some(Format::Wbfs),
        "wia" => Some(Format::Wia),
        "tgc" => Some(Format::Tgc),
        _ => None,
    }
}

fn perform_blocking(
    game: &Game,
    out_path: &Path,
    cancel: &ExportCancellation,
    tx: &smol::channel::Sender<LongOperationState>,
) -> Result<()> {
    cancel.check()?;
    let disc_path = game.get_disc_path_blocking().context("disc not found")?;
    cancel.check()?;
    if out_path.exists()
        && std::fs::canonicalize(out_path)?.starts_with(std::fs::canonicalize(game.path())?)
    {
        bail!("cannot replace a file in the source game folder");
    }
    let format = out_path
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ext_to_format)
        .context("invalid extension")?;
    let mut previous_percentage = None;
    export_disc(
        &disc_path,
        out_path,
        DiscExportOptions {
            format,
            threads: ExportThreads::for_jobs(1),
            destination: DestinationMode::ReplaceConfirmed,
        },
        cancel,
        |percentage| {
            if previous_percentage != Some(percentage) {
                let _ = tx.try_send(LongOperationState::Progress(format!(
                    "⤓  Exporting {}  {percentage:02}%",
                    game.title()
                )));
                previous_percentage = Some(percentage);
            }
            Ok(())
        },
    )
}

pub fn export_game(
    game: Game,
    out_path: PathBuf,
    cancel: ExportCancellation,
) -> impl Stream<Item = LongOperationState> {
    let (tx, rx) = smol::channel::bounded(1);
    let _ = std::thread::spawn(move || {
        let cancel = cancel.observed_by(&tx);
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match perform_blocking(&game, &out_path, &cancel, &tx) {
                    Ok(()) => LongOperationState::Finished(format!("Exported {}", game.title())),
                    Err(error) if error_contains::<DiscCancelled>(error.as_ref()) => {
                        LongOperationState::Cancelled(format!(
                            "Export cancelled: {}. {error:#}",
                            game.title()
                        ))
                    }
                    Err(error) => LongOperationState::Errored(format!("{error:#}")),
                }
            }))
            .unwrap_or_else(|_| {
                LongOperationState::Errored(
                    "An export worker panicked; incomplete output was not published".into(),
                )
            });
        let _ = tx.send_blocking(result);
    });
    rx
}

fn output_filename(game: &Game, disc: Option<u16>, format: ExportFormat) -> String {
    let name = make_game_dir_name(game.id(), game.title());
    let suffix = disc
        .map(|number| format!(" (Disc {number})"))
        .unwrap_or_default();
    format!("{name}{suffix}.{}", format.extension())
}

// Publish only a fully written image. Single export may replace a file after the
// save dialog confirmed it; cancellation must leave that old file untouched.
fn write_staged(
    out_path: &Path,
    cancel: &ExportCancellation,
    mode: DestinationMode,
    write: impl FnOnce(&mut BufWriter<CancellableWriter<'_>>) -> Result<()>,
) -> Result<()> {
    cancel.check()?;
    if matches!(mode, DestinationMode::KeepExisting) {
        match std::fs::symlink_metadata(out_path) {
            Ok(_) => bail!("destination already exists: {}", out_path.display()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error).context("checking destination"),
        }
    }

    let parent = out_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::Builder::new()
        .prefix(".twbm-export-")
        .tempfile_in(parent)
        .context("creating temporary export file")?;
    let written = (|| -> Result<()> {
        let mut writer = BufWriter::with_capacity(
            1024 * 1024,
            CancellableWriter {
                file: temporary.as_file_mut(),
                cancel,
            },
        );
        write(&mut writer)?;
        cancel.check()?;
        writer.flush()?;
        drop(writer);
        cancel.check()?;
        temporary.as_file().sync_all()?;
        cancel.check()?;
        Ok(())
    })();
    if let Err(error) = written {
        return Err(discard_temporary(temporary, error));
    }
    // The rename itself is the atomic commit point and cannot be interrupted.
    // A cancellation racing after it keeps this already completed image.
    let saved = match mode {
        DestinationMode::KeepExisting => temporary.persist_noclobber(out_path),
        DestinationMode::ReplaceConfirmed => temporary.persist(out_path),
    };
    if let Err(error) = saved {
        return Err(discard_temporary(error.file, error.error.into()))
            .with_context(|| format!("saving {}", out_path.display()));
    }
    Ok(())
}

fn discard_temporary(temporary: tempfile::NamedTempFile, error: anyhow::Error) -> anyhow::Error {
    let path = temporary.path().to_path_buf();
    match temporary.close() {
        Ok(()) => error,
        Err(cleanup) => error.context(format!(
            "Could not remove incomplete export {}: {cleanup}",
            path.display()
        )),
    }
}

fn export_disc(
    source: &Path,
    destination: &Path,
    options: DiscExportOptions,
    cancel: &ExportCancellation,
    mut report_progress: impl FnMut(u64) -> Result<()>,
) -> Result<()> {
    cancel.check()?;
    // Do not replace the source through the individual save dialog, including a
    // symlink to it. Batch destinations never replace existing files at all.
    if matches!(options.destination, DestinationMode::ReplaceConfirmed)
        && destination.exists()
        && std::fs::canonicalize(source)? == std::fs::canonicalize(destination)?
    {
        bail!("export destination is the source image");
    }
    write_staged(destination, cancel, options.destination, |output| {
        report_progress(0)?;
        let files =
            retrieve_multi_disc_files_cancellable(source, cancel).context("opening source disc")?;
        let stream = CancellableDiscStream::new(SharedMultiFileReader::new(files)?, cancel.clone());
        let reader = DiscReader::new_stream(
            Box::new(stream.clone()),
            &DiscOptions {
                preloader_threads: options.threads.preloader,
                ..Default::default()
            },
        )?;
        cancel.check()?;
        let writer = DiscWriter::new(reader, &FormatOptions::new(options.format))?;
        cancel.check()?;
        let finalization = writer.process(
            |data, progress, total| {
                let percentage = progress.saturating_mul(100).checked_div(total).unwrap_or(0);
                // Check cancellation before every write, including chunks that
                // do not change the displayed percentage.
                cancel.check()?;
                report_progress(percentage).map_err(|error| {
                    if error_contains::<DiscCancelled>(error.as_ref()) {
                        io::Error::other(DiscCancelled("Export"))
                    } else {
                        io::Error::other(error)
                    }
                })?;
                output.write_all(&data)?;
                Ok(())
            },
            &ProcessOptions {
                processor_threads: options.threads.processor,
                scrub: ScrubLevel::None,
                digest_crc32: true,
                digest_md5: false,
                digest_sha1: true,
                digest_xxh64: true,
            },
        )?;
        cancel.check()?;
        if !finalization.header.is_empty() {
            output.rewind()?;
            output.write_all(&finalization.header)?;
        }
        report_progress(100)?;
        cancel.check()?;
        Ok(())
    })
}

#[derive(Default)]
struct ExportProgress {
    completed: usize,
    active: BTreeMap<usize, String>,
}

impl ExportProgress {
    fn publish(&self, total: usize, tx: &smol::channel::Sender<LongOperationState>) {
        let details = self.active.values().cloned().collect::<Vec<_>>().join("\n");
        // Never block a worker on the UI, especially while holding the mutex.
        let _ = tx.try_send(LongOperationState::Progress(format!(
            "⤓  Exporting: {}/{total} completed, {} active\n{details}",
            self.completed,
            self.active.len()
        )));
    }
}

#[derive(Default)]
struct GameExportResult {
    discs: usize,
    failures: Vec<String>,
    failed: bool,
}

// Each worker claims one game at a time. Results retain the selection order,
// regardless of completion order. The caller also works, so a failed helper
// thread spawn only reduces concurrency rather than abandoning the export.
fn run_jobs<T: Send>(
    count: usize,
    parallel_jobs: usize,
    run: impl Fn(usize) -> T + Sync,
) -> Result<Vec<T>> {
    let next = AtomicUsize::new(0);
    let work = || {
        let mut results = Vec::new();
        loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            if index >= count {
                break;
            }
            results.push((index, run(index)));
        }
        results
    };
    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for index in 1..parallel_jobs {
            match std::thread::Builder::new()
                .name(format!("game-export-{index}"))
                .spawn_scoped(scope, work)
            {
                Ok(handle) => handles.push(handle),
                Err(_) => break,
            }
        }
        let mut results = work();
        let mut panicked = false;
        for handle in handles {
            match handle.join() {
                Ok(completed) => results.extend(completed),
                Err(_) => panicked = true,
            }
        }
        if panicked {
            bail!("An export worker panicked; completed files have been kept");
        }
        results.sort_unstable_by_key(|(index, _)| *index);
        Ok(results.into_iter().map(|(_, result)| result).collect())
    })
}

fn perform_batch_blocking(
    games: &[Game],
    destination: &Path,
    format: ExportFormat,
    parallel_jobs: usize,
    cancel: &ExportCancellation,
    tx: &smol::channel::Sender<LongOperationState>,
) -> LongOperationState {
    if tx.is_closed() {
        cancel.cancel();
    }
    if cancel.is_cancelled() {
        return LongOperationState::Cancelled("Export cancelled before starting".into());
    }
    if games.is_empty() {
        return LongOperationState::Errored("No games selected for export".to_string());
    }
    if !destination.is_dir() {
        return LongOperationState::Errored(format!(
            "Export folder is unavailable: {}",
            destination.display()
        ));
    }

    let jobs = parallel_jobs
        .clamp(1, MAX_PARALLEL_EXPORTS)
        .min(games.len());
    let threads = ExportThreads::for_jobs(jobs);
    let progress = Mutex::new(ExportProgress::default());
    let results = run_jobs(games.len(), jobs, |index| {
        // The receiver disappearing means the operation is no longer observed.
        // Do not open more source images or create more output files after that.
        if cancel.is_cancelled() || tx.is_closed() {
            return None;
        }
        let game = &games[index];
        let label = format!("{} [{}]", game.title(), game.id());
        {
            let mut progress = progress.lock().unwrap();
            progress.active.insert(index, label.clone());
            progress.publish(games.len(), tx);
        }
        let mut result = GameExportResult::default();
        let discs = match game.get_disc_paths_blocking() {
            Ok(discs) => discs,
            Err(error) => {
                result.failed = true;
                result.failures.push(format!("{label}: {error:#}"));
                Vec::new()
            }
        };
        let is_multidisc = discs.len() > 1;
        for (number, source) in discs {
            if cancel.is_cancelled() || tx.is_closed() {
                result.failures.push(format!("{label}: export cancelled"));
                break;
            }
            let filename = output_filename(game, is_multidisc.then_some(number), format);
            let label = if is_multidisc {
                format!("{label} (Disc {number})")
            } else {
                label.clone()
            };
            let mut previous_percentage = None;
            let report = |percentage| {
                cancel.check()?;
                if previous_percentage != Some(percentage) {
                    let mut progress = progress.lock().unwrap();
                    progress
                        .active
                        .insert(index, format!("{label}  {percentage:02}%"));
                    progress.publish(games.len(), tx);
                    previous_percentage = Some(percentage);
                }
                Ok(())
            };
            match export_disc(
                &source,
                &destination.join(&filename),
                DiscExportOptions::batch(format, threads),
                cancel,
                report,
            ) {
                Ok(()) => result.discs += 1,
                Err(error) => {
                    if !error_contains::<DiscCancelled>(error.as_ref()) {
                        result.failed = true;
                    }
                    result.failures.push(format!("{filename}: {error:#}"));
                }
            }
        }
        {
            let mut progress = progress.lock().unwrap();
            progress.active.remove(&index);
            progress.completed += 1;
            progress.publish(games.len(), tx);
        }
        Some(result)
    });
    let results = match results {
        Ok(results) => results,
        Err(error) => return LongOperationState::Errored(error.to_string()),
    };
    let exported_games = results
        .iter()
        .flatten()
        .filter(|result| result.failures.is_empty())
        .count();
    let exported_discs: usize = results.iter().flatten().map(|result| result.discs).sum();
    let failed_games = results
        .iter()
        .flatten()
        .filter(|result| result.failed)
        .count();
    let has_real_failures = failed_games > 0;
    let failures: Vec<_> = results
        .into_iter()
        .flatten()
        .flat_map(|result| result.failures)
        .collect();

    let summary = format!(
        "Exported {exported_games} of {} games ({exported_discs} disc images)",
        games.len()
    );
    if !has_real_failures
        && (cancel.is_cancelled() || tx.is_closed())
        && exported_games != games.len()
    {
        return LongOperationState::Cancelled(format!(
            "Export cancelled; {summary}. Completed images kept.{}",
            if failures.is_empty() {
                String::new()
            } else {
                format!("\n{}", failures.join("\n"))
            }
        ));
    }
    if failures.is_empty() {
        LongOperationState::Finished(summary)
    } else {
        LongOperationState::Errored(format!(
            "{summary}; {failed_games} games failed.{}\n{}",
            if games.len() > exported_games + failed_games {
                format!(
                    " {} games cancelled or not started.",
                    games.len() - exported_games - failed_games
                )
            } else {
                String::new()
            },
            failures.join("\n")
        ))
    }
}

pub fn export_games(
    games: Vec<Game>,
    destination: PathBuf,
    format: ExportFormat,
    parallel_jobs: usize,
    cancel: ExportCancellation,
) -> impl Stream<Item = LongOperationState> {
    let (tx, rx) = smol::channel::bounded(1);
    let _ = std::thread::spawn(move || {
        let cancel = cancel.observed_by(&tx);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            perform_batch_blocking(&games, &destination, format, parallel_jobs, &cancel, &tx)
        }))
        .unwrap_or_else(|_| {
            LongOperationState::Errored(
                "An export worker panicked; completed files have been kept".into(),
            )
        });
        // All workers are joined before this single terminal event makes the UI idle.
        let _ = tx.send_blocking(result);
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::stream::StreamExt;
    use std::io::Read;

    fn game(base: &Path, title: &str, id: &str) -> Game {
        let path = base.join(format!("{title} [{id}]"));
        std::fs::create_dir(&path).unwrap();
        smol::block_on(Game::try_from_path(path, false)).unwrap()
    }

    // Minimal synthetic disc data: a GameCube header and zero-filled sectors.
    fn synthetic_disc() -> Vec<u8> {
        let mut data = vec![0; 0x10000];
        data[..6].copy_from_slice(b"GZZE01");
        data[0x1c..0x20].copy_from_slice(&[0xc2, 0x33, 0x9f, 0x3d]);
        data
    }

    #[test]
    fn filenames_are_safe_and_identify_each_disc() {
        let temp = tempfile::tempdir().unwrap();
        let game = game(temp.path(), "Example", "GZZE01");
        assert_eq!(
            output_filename(&game, None, ExportFormat::Rvz),
            "Example [GZZE01].rvz"
        );
        assert_eq!(
            output_filename(&game, Some(2), ExportFormat::Iso),
            "Example [GZZE01] (Disc 2).iso"
        );
        let sanitized = make_game_dir_name(game.id(), "../Bad: Game?*");
        assert_eq!(sanitized, "Bad Game [GZZE01]");
    }

    #[test]
    fn staging_preserves_existing_destination_and_source() {
        let temp = tempfile::tempdir().unwrap();
        for name in ["source.iso", "existing.iso"] {
            let path = temp.path().join(name);
            std::fs::write(&path, b"keep this data").unwrap();
            let error = write_staged(
                &path,
                &ExportCancellation::default(),
                DestinationMode::KeepExisting,
                |_| panic!("must reject before writing"),
            )
            .unwrap_err();
            assert!(error.to_string().contains("already exists"));
            assert_eq!(std::fs::read(&path).unwrap(), b"keep this data");
        }
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn staging_cleans_up_partial_writes_and_handles_destination_races() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("output.iso");
        let error = write_staged(
            &path,
            &ExportCancellation::default(),
            DestinationMode::KeepExisting,
            |output| {
                output.write_all(b"partial")?;
                bail!("simulated write failure")
            },
        );
        assert!(error.is_err());
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);

        let error = write_staged(
            &path,
            &ExportCancellation::default(),
            DestinationMode::KeepExisting,
            |output| {
                output.write_all(b"new export")?;
                std::fs::write(&path, b"created meanwhile")?;
                Ok(())
            },
        );
        assert!(error.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"created meanwhile");
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn batch_continues_after_failure_and_exports_both_discs() {
        let temp = tempfile::tempdir().unwrap();
        let missing = game(temp.path(), "Missing", "GZZP01");
        let valid = game(temp.path(), "Two discs", "GZZE01");
        let data = synthetic_disc();
        std::fs::write(valid.path().join("game.iso"), &data).unwrap();
        std::fs::write(valid.path().join("disc2.iso"), &data).unwrap();
        let destination = temp.path().join("export");
        std::fs::create_dir(&destination).unwrap();
        let states = smol::block_on(
            export_games(
                vec![missing, valid.clone()],
                destination.clone(),
                ExportFormat::Iso,
                DEFAULT_PARALLEL_EXPORTS,
                ExportCancellation::default(),
            )
            .collect::<Vec<_>>(),
        );
        assert_eq!(
            states
                .iter()
                .filter(|state| matches!(
                    state,
                    LongOperationState::Finished(_) | LongOperationState::Errored(_)
                ))
                .count(),
            1,
            "individual failures must not terminate the batch stream"
        );
        match states.last().unwrap() {
            LongOperationState::Errored(summary) => {
                assert!(summary.contains("Exported 1 of 2 games (2 disc images)"));
                assert!(summary.contains("1 games failed"));
                assert!(summary.contains("disc not found"));
            }
            state => panic!("unexpected final state: {state:?}"),
        }
        for number in [1, 2] {
            assert_eq!(
                std::fs::read(destination.join(output_filename(
                    &valid,
                    Some(number),
                    ExportFormat::Iso
                )))
                .unwrap(),
                data
            );
        }
        assert_eq!(std::fs::read(valid.path().join("game.iso")).unwrap(), data);
        assert_eq!(std::fs::read_dir(destination).unwrap().count(), 2);
    }

    #[test]
    fn default_rvz_export_round_trips_synthetic_disc() {
        let temp = tempfile::tempdir().unwrap();
        let game = game(temp.path(), "Compressed", "GZZE01");
        let mut data = synthetic_disc();
        // Exercise several RVZ chunks, including a final partial chunk.
        data.resize(0x50000, 0);
        for (index, byte) in data[0x8000..].iter_mut().enumerate() {
            *byte = (index % 251) as u8;
        }
        let source = game.path().join("game.iso");
        std::fs::write(&source, &data).unwrap();
        let destination = temp.path().join("export");
        std::fs::create_dir(&destination).unwrap();
        let format = ExportFormat::default();
        assert_eq!(format, ExportFormat::Rvz);

        let states = smol::block_on(
            export_games(
                vec![game.clone()],
                destination.clone(),
                format,
                DEFAULT_PARALLEL_EXPORTS,
                ExportCancellation::default(),
            )
            .collect::<Vec<_>>(),
        );
        assert!(
            matches!(states.last(), Some(LongOperationState::Finished(_))),
            "unexpected export states: {states:?}"
        );
        let output = destination.join(output_filename(&game, None, format));
        let mut reader = DiscReader::new(output, &DiscOptions::default()).unwrap();
        assert_eq!(reader.meta().format, Format::Rvz);
        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, data);
        assert_eq!(std::fs::read(source).unwrap(), data);
        assert_eq!(std::fs::read_dir(destination).unwrap().count(), 1);
    }

    #[test]
    fn batch_joins_split_images_and_rejects_output_collisions() {
        let temp = tempfile::tempdir().unwrap();
        let game = game(temp.path(), "Split", "GZZE01");
        let data = synthetic_disc();
        std::fs::write(game.path().join("GZZE01.part0.iso"), &data[..0x8000]).unwrap();
        std::fs::write(game.path().join("GZZE01.part1.iso"), &data[0x8000..]).unwrap();
        let destination = temp.path().join("export");
        std::fs::create_dir(&destination).unwrap();
        let (tx, _rx) = smol::channel::unbounded();
        let games = [game.clone()];
        assert!(matches!(
            perform_batch_blocking(
                &games,
                &destination,
                ExportFormat::Iso,
                DEFAULT_PARALLEL_EXPORTS,
                &ExportCancellation::default(),
                &tx
            ),
            LongOperationState::Finished(_)
        ));
        let output = destination.join(output_filename(&game, None, ExportFormat::Iso));
        assert_eq!(std::fs::read(&output).unwrap(), data);
        match perform_batch_blocking(
            &games,
            &destination,
            ExportFormat::Iso,
            DEFAULT_PARALLEL_EXPORTS,
            &ExportCancellation::default(),
            &tx,
        ) {
            LongOperationState::Errored(summary) => assert!(summary.contains("already exists")),
            state => panic!("unexpected final state: {state:?}"),
        }
        assert_eq!(std::fs::read(output).unwrap(), data);
        assert_eq!(std::fs::read_dir(destination).unwrap().count(), 1);
    }

    #[test]
    fn workers_overlap_respect_the_limit_and_finish_every_job() {
        use std::{sync::mpsc, time::Duration};
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        std::thread::scope(|scope| {
            let runner = scope.spawn(|| {
                run_jobs(12, 3, |index| {
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(current, Ordering::SeqCst);
                    started_tx.send(index).unwrap();
                    release_rx
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(10))
                        .unwrap();
                    active.fetch_sub(1, Ordering::SeqCst);
                    index
                })
                .unwrap()
            });
            for _ in 0..3 {
                started_rx.recv_timeout(Duration::from_secs(10)).unwrap();
            }
            assert_eq!(active.load(Ordering::SeqCst), 3);
            for _ in 0..12 {
                release_tx.send(()).unwrap();
            }
            assert_eq!(runner.join().unwrap(), (0..12).collect::<Vec<_>>());
        });
        assert_eq!(peak.load(Ordering::SeqCst), 3);
        assert_eq!(active.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn simultaneous_writers_never_mix_or_overwrite_the_same_output() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("collision.iso");
        let barrier = std::sync::Barrier::new(2);
        let results = run_jobs(2, 2, |index| {
            write_staged(
                &path,
                &ExportCancellation::default(),
                DestinationMode::KeepExisting,
                |output| {
                    output.write_all(&vec![index as u8; 0x10000])?;
                    barrier.wait();
                    Ok(())
                },
            )
        })
        .unwrap();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        let data = std::fs::read(path).unwrap();
        assert_eq!(data.len(), 0x10000);
        assert!(data.iter().all(|&byte| byte == data[0]));
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn cancelled_conversion_removes_partial_output_and_keeps_source() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.iso");
        let destination = temp.path().join("output.rvz");
        let mut data = synthetic_disc();
        data.resize(0x80000, 0);
        std::fs::write(&source, &data).unwrap();
        let error = export_disc(
            &source,
            &destination,
            DiscExportOptions::batch(ExportFormat::Rvz, ExportThreads::for_jobs(2)),
            &ExportCancellation::default(),
            |percentage| {
                if percentage > 0 {
                    bail!("test cancellation");
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert!(format!("{error:#}").contains("test cancellation"));
        assert!(!destination.exists());
        assert_eq!(std::fs::read(source).unwrap(), data);
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn closed_receiver_does_not_start_more_exports() {
        let temp = tempfile::tempdir().unwrap();
        let game = game(temp.path(), "Cancelled", "GZZE01");
        std::fs::write(game.path().join("game.iso"), synthetic_disc()).unwrap();
        let destination = temp.path().join("export");
        std::fs::create_dir(&destination).unwrap();
        let (tx, rx) = smol::channel::bounded(1);
        drop(rx);
        let result = perform_batch_blocking(
            &[game],
            &destination,
            ExportFormat::Rvz,
            2,
            &ExportCancellation::default(),
            &tx,
        );
        assert!(matches!(result, LongOperationState::Cancelled(_)));
        assert_eq!(std::fs::read_dir(destination).unwrap().count(), 0);
    }

    #[test]
    fn parallel_exports_preserve_distinct_disc_contents_in_every_format() {
        let temp = tempfile::tempdir().unwrap();
        let mut games = Vec::new();
        let mut originals = Vec::new();
        for index in 0..4 {
            let game = game(temp.path(), &format!("Parallel {index}"), "GZZE01");
            let mut data = synthetic_disc();
            data.resize(0x50000, 0);
            for (offset, byte) in data[0x8000..].iter_mut().enumerate() {
                *byte = ((offset * (index + 1) + index * 47) % 251) as u8;
            }
            std::fs::write(game.path().join("game.iso"), &data).unwrap();
            games.push(game);
            originals.push(data);
        }
        for format in ExportFormat::ALL {
            let destination = temp.path().join(format.extension());
            std::fs::create_dir(&destination).unwrap();
            let states = smol::block_on(
                export_games(
                    games.clone(),
                    destination.clone(),
                    format,
                    4,
                    ExportCancellation::default(),
                )
                .collect::<Vec<_>>(),
            );
            assert!(
                matches!(states.last(), Some(LongOperationState::Finished(summary)) if summary.contains("4 of 4 games (4 disc images)")),
                "{format}: {states:?}"
            );
            assert_eq!(
                states
                    .iter()
                    .filter(|state| matches!(
                        state,
                        LongOperationState::Finished(_) | LongOperationState::Errored(_)
                    ))
                    .count(),
                1
            );
            for (game, expected) in games.iter().zip(&originals) {
                let output = destination.join(output_filename(game, None, format));
                let reader = DiscReader::new(output, &DiscOptions::default()).unwrap();
                let mut decoded = Vec::new();
                reader
                    .take(expected.len() as u64)
                    .read_to_end(&mut decoded)
                    .unwrap();
                assert_eq!(&decoded, expected, "{format}: {}", game.title());
                assert_eq!(
                    std::fs::read(game.path().join("game.iso")).unwrap(),
                    *expected
                );
            }
            assert_eq!(std::fs::read_dir(destination).unwrap().count(), 4);
        }
    }
}
