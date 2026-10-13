// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::Config,
    games::{
        ImportEntry,
        disc_reader::{
            CancellableDiscStream, DiscCancellation, DiscCancelled, SharedMultiFileReader,
            error_contains, retrieve_multi_disc_files_cancellable,
        },
    },
    util::{drive_state::DriveState, misc::OPTIMAL_THREADS},
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
    fs::File,
    io::{self, BufWriter, Read, Seek, Write},
    num::NonZeroUsize,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};
use which_fs::FsKind;
use wii_disc_info::game_id::GameID;

mod staging;
use staging::StagedWriter;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod cancel_tests;

pub const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB
pub const MAX_PARALLEL_IMPORTS: usize = 4;
pub const DEFAULT_PARALLEL_IMPORTS: usize = 2;

#[derive(Debug, Clone, Default)]
pub struct ImportCancellation {
    requested: Arc<AtomicBool>,
    observer: Option<smol::channel::Sender<BatchImportEvent>>,
}

#[derive(Debug)]
struct CleanupWarning(String);

impl std::fmt::Display for CleanupWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for CleanupWarning {}

impl ImportCancellation {
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

    fn observed_by(mut self, sender: &smol::channel::Sender<BatchImportEvent>) -> Self {
        self.observer = Some(sender.clone());
        self
    }
}

impl DiscCancellation for ImportCancellation {
    fn check(&self) -> io::Result<()> {
        // Interrupted would be retried forever by read_exact/write_all.
        if self.is_cancelled() {
            Err(io::Error::other(DiscCancelled("Import")))
        } else {
            Ok(())
        }
    }
}

fn is_cancel_error(error: &anyhow::Error) -> bool {
    error_contains::<DiscCancelled>(error.as_ref())
}

fn callback_io_error(error: anyhow::Error) -> io::Error {
    // Boxing anyhow inside io::Error can erase the root error's downcast type.
    // Preserve the cancellation marker explicitly at this API boundary.
    if is_cancel_error(&error) {
        io::Error::other(DiscCancelled("Import"))
    } else {
        io::Error::other(error)
    }
}

#[derive(Debug, Clone)]
pub enum BatchImportEvent {
    Progress(String),
    Result(ImportResult),
    Finished(ImportSummary),
}

#[derive(Debug, Clone)]
pub struct ImportResult {
    pub entry: ImportEntry,
    pub error: Option<String>,
    pub warning: Option<String>,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportSummary {
    pub imported: usize,
    pub failed: usize,
    pub cancelled: usize,
}

impl std::fmt::Display for ImportSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Imported {} of {} games",
            self.imported,
            self.imported + self.failed + self.cancelled
        )?;
        if self.failed > 0 {
            write!(f, "; {} failed", self.failed)?;
        }
        if self.cancelled > 0 {
            write!(f, "; {} cancelled", self.cancelled)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct ImportThreads {
    preloader: usize,
    processor: usize,
}

impl ImportThreads {
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
            processor: (cores / jobs).saturating_sub(4).min(1),
        }
    }
}

struct ImportPlan {
    folder: PathBuf,
    format: Format,
    split_size: Option<NonZeroUsize>,
}

impl ImportPlan {
    fn new(entry: &ImportEntry, config: &Config, drive: &DriveState) -> Self {
        let is_fat32 = match drive {
            DriveState::Loaded(info) => info.fs_kind() == FsKind::Fat32,
            _ => true,
        };
        Self {
            folder: config
                .mount_point()
                .join(if entry.meta().is_wii() {
                    "wbfs"
                } else {
                    "games"
                })
                .join(make_game_dir_name(
                    entry.meta().game_id(),
                    entry.meta().game_title(),
                )),
            format: if entry.meta().is_wii() {
                config.wii_output_format()
            } else {
                config.gc_output_format()
            },
            split_size: (entry.meta().is_wii() && (is_fat32 || config.always_split()))
                .then_some(SPLIT_SIZE),
        }
    }

    fn filename(&self, entry: &ImportEntry, config: &Config, part: usize) -> String {
        get_filename(
            entry.meta().game_id(),
            entry.meta().is_wii(),
            part,
            entry.meta().disc_number(),
            config.wii_output_format(),
            config.gc_output_format(),
            self.split_size.is_some(),
        )
    }

    fn ensure_available(&self, entry: &ImportEntry, cancel: &ImportCancellation) -> Result<()> {
        cancel.check()?;
        let files = match std::fs::read_dir(&self.folder) {
            Ok(files) => files,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error).context("Cannot inspect destination game folder"),
        };
        let prefix = if entry.meta().is_wii() {
            format!("{}.", entry.meta().game_id()).to_ascii_lowercase()
        } else if entry.meta().disc_number() == 0 {
            "game.".into()
        } else {
            format!("disc{}.", u16::from(entry.meta().disc_number()) + 1)
        };
        for file in files {
            cancel.check()?;
            let file = file?;
            if file
                .file_name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .starts_with(&prefix)
            {
                bail!(
                    "A file for this disc already exists: {}",
                    file.path().display()
                );
            }
        }
        Ok(())
    }
}

fn write_output<W: Write, T>(
    mut writer: W,
    cancel: &ImportCancellation,
    produce: impl FnOnce(&mut dyn Write) -> Result<T>,
) -> (W, Result<T>) {
    let result = (|| {
        cancel.check()?;
        let value = produce(&mut writer)?;
        cancel.check()?;
        writer.flush()?;
        cancel.check()?;
        Ok(value)
    })();
    (writer, result)
}

fn copy_files<C: DiscCancellation>(
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
    let mut buffer = vec![0; 1024 * 1024];
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

fn perform_blocking(
    entry: &ImportEntry,
    config: &Config,
    plan: &ImportPlan,
    threads: ImportThreads,
    commit: &Mutex<()>,
    cancel: &ImportCancellation,
    mut report_progress: impl FnMut(u64) -> Result<()>,
) -> Result<Option<String>> {
    let mut report = |percentage| -> Result<()> {
        cancel.check()?;
        report_progress(percentage)?;
        cancel.check()?;
        Ok(())
    };
    report(0)?;
    plan.ensure_available(entry, cancel)?;
    let mut files = retrieve_multi_disc_files_cancellable(entry.path(), cancel)
        .context("Cannot open source image")?;
    report(0)?;
    let parent = plan.folder.parent().context("Invalid game folder")?;
    std::fs::create_dir_all(parent)?;
    let staged = StagedWriter::new(
        parent,
        |part| plan.filename(entry, config, part),
        plan.split_size,
        cancel.clone(),
    )?;
    let output = BufWriter::with_capacity(1024 * 1024, staged);
    let (output, converted) = write_output(output, cancel, |output| -> Result<Vec<u8>> {
        let header = if plan.format == wii_disc_info_to_nod_format(entry.meta().format())
            && !(entry.meta().is_wii() && config.scrub_update_partition())
        {
            copy_files(&mut files, output, cancel, &mut report)?;
            Vec::new()
        } else {
            let stream =
                CancellableDiscStream::new(SharedMultiFileReader::new(files)?, cancel.clone());
            cancel.check()?;
            let reader = DiscReader::new_stream(
                Box::new(stream),
                &DiscOptions {
                    preloader_threads: threads.preloader,
                    ..Default::default()
                },
            )?;
            cancel.check()?;
            let writer = DiscWriter::new(reader, &FormatOptions::new(plan.format))?;
            cancel.check()?;
            let finalization = writer.process(
                |data, progress, total| {
                    report(progress.saturating_mul(100).checked_div(total).unwrap_or(0))
                        .map_err(callback_io_error)?;
                    output.write_all(&data)?;
                    Ok(())
                },
                &ProcessOptions {
                    processor_threads: threads.processor,
                    scrub: if config.scrub_update_partition() {
                        ScrubLevel::UpdatePartition
                    } else {
                        ScrubLevel::None
                    },
                    digest_crc32: true,
                    digest_md5: false,
                    digest_sha1: true,
                    digest_xxh64: true,
                },
            )?;
            cancel.check()?;
            finalization.header.to_vec()
        };
        cancel.check()?;
        Ok(header)
    });
    // Discard buffered bytes on error; never flush more data while cancelling.
    let (writer, _) = output.into_parts();
    let header = match converted {
        Ok(header) => header,
        Err(error) => return Err(writer.discard(error)),
    };
    // The output buffer is flushed; finish syncs and closes every
    // part before publication, including same-format WBFS copies (upstream #694).
    let staged = writer
        .finish(&header)
        .context("Cannot finalize imported image")?;
    if let Err(error) = report(100) {
        return Err(staged.discard(error));
    }
    let mut warnings = Vec::new();
    {
        // Conversion stays parallel. Only the short publication transaction is
        // serialized, including GC discs sharing the same destination folder.
        let _commit = commit
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ready = report(100).and_then(|()| plan.ensure_available(entry, cancel));
        if let Err(error) = ready {
            return Err(staged.discard(error));
        }
        if let Some(warning) = staged.publish(&plan.folder, cancel)? {
            warnings.push(warning);
        }
    }
    if config.remove_sources_games()
        && !cancel.is_cancelled()
        && let Err(error) = std::fs::remove_file(entry.path())
    {
        warnings.push(format!(
            "Image imported successfully, but the source could not be removed: {error}"
        ));
    }
    Ok((!warnings.is_empty()).then(|| warnings.join("\n")))
}

#[derive(Default)]
struct ImportProgress {
    completed: usize,
    active: BTreeMap<usize, String>,
}

impl ImportProgress {
    fn publish(&self, total: usize, tx: &smol::channel::Sender<BatchImportEvent>) {
        let details = self.active.values().cloned().collect::<Vec<_>>().join("\n");
        let _ = tx.try_send(BatchImportEvent::Progress(format!(
            "⤓  Importing: {}/{total} completed, {} active\n{details}",
            self.completed,
            self.active.len()
        )));
    }
}

fn run_jobs<T: Send>(
    count: usize,
    parallel_jobs: usize,
    stopped: impl Fn() -> bool + Sync,
    run: impl Fn(usize) -> T + Sync,
) -> Vec<T> {
    let next = AtomicUsize::new(0);
    let work = || {
        let mut results = Vec::new();
        while !stopped() {
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
        for index in 1..parallel_jobs.clamp(1, MAX_PARALLEL_IMPORTS).min(count) {
            match std::thread::Builder::new()
                .name(format!("game-import-{index}"))
                .spawn_scoped(scope, work)
            {
                Ok(handle) => handles.push(handle),
                Err(_) => break, // The caller still processes all queued games.
            }
        }
        let mut results = work();
        for handle in handles {
            if let Ok(completed) = handle.join() {
                results.extend(completed);
            }
        }
        results.sort_unstable_by_key(|(index, _)| *index);
        results.into_iter().map(|(_, result)| result).collect()
    })
}

fn preflight(
    entries: &[ImportEntry],
    config: &Config,
    drive: &DriveState,
    cancel: &ImportCancellation,
) -> Vec<Result<ImportPlan>> {
    let mut plans: Vec<_> = entries
        .iter()
        .map(|entry| {
            cancel.check()?;
            if config.mount_point().as_os_str().is_empty() || !config.mount_point().is_dir() {
                bail!("The destination drive is unavailable");
            }
            let plan = ImportPlan::new(entry, config, drive);
            if (entry.meta().is_wii() && !matches!(plan.format, Format::Iso | Format::Wbfs))
                || (entry.meta().is_gc() && !matches!(plan.format, Format::Iso | Format::Ciso))
            {
                bail!(
                    "Unsupported output format for this console: {}",
                    plan.format
                );
            }
            plan.ensure_available(entry, cancel)?;
            Ok(plan)
        })
        .collect();
    let mut identities = BTreeMap::<_, Vec<usize>>::new();
    for (index, entry) in entries.iter().enumerate() {
        if cancel.is_cancelled() {
            break;
        }
        let key = (
            entry.meta().is_wii(),
            entry.meta().game_id().to_string(),
            if entry.meta().is_wii() {
                0
            } else {
                entry.meta().disc_number()
            },
        );
        identities.entry(key).or_default().push(index);
    }
    for duplicates in identities.values().filter(|indices| indices.len() > 1) {
        if cancel.is_cancelled() {
            break;
        }
        for &index in duplicates {
            plans[index] = Err(anyhow::anyhow!(
                "Multiple selected images target the same game and disc; choose one source"
            ));
        }
    }
    plans
}

pub fn import_games(
    entries: Vec<ImportEntry>,
    config: Config,
    drive: DriveState,
    parallel_jobs: usize,
    cancellation: ImportCancellation,
) -> impl Stream<Item = BatchImportEvent> {
    let (tx, rx) = smol::channel::bounded(MAX_PARALLEL_IMPORTS * 2 + 1);
    let _ = std::thread::spawn(move || {
        let cancellation = cancellation.observed_by(&tx);
        let plans = preflight(&entries, &config, &drive, &cancellation);
        let jobs = parallel_jobs
            .clamp(1, MAX_PARALLEL_IMPORTS)
            .min(entries.len().max(1));
        let threads = ImportThreads::for_jobs(jobs);
        let progress = Mutex::new(ImportProgress::default());
        let commit = Mutex::new(());
        let results = run_jobs(
            entries.len(),
            jobs,
            || cancellation.is_cancelled(),
            |index| {
                let entry = &entries[index];
                let label = format!("{} [{}]", entry.meta().game_title(), entry.meta().game_id());
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || -> Result<Option<String>> {
                        cancellation.check()?;
                        let plan = plans[index]
                            .as_ref()
                            .map_err(|error| anyhow::anyhow!("{error:#}"))?;
                        let mut previous = None;
                        perform_blocking(
                            entry,
                            &config,
                            plan,
                            threads,
                            &commit,
                            &cancellation,
                            |percentage| {
                                if previous != Some(percentage) {
                                    let mut state = progress
                                        .lock()
                                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                                    state
                                        .active
                                        .insert(index, format!("{label}  {percentage:02}%"));
                                    state.publish(entries.len(), &tx);
                                    previous = Some(percentage);
                                }
                                Ok(())
                            },
                        )
                    },
                ))
                .unwrap_or_else(|_| {
                    Err(anyhow::anyhow!(
                        "Import worker panicked; incomplete output was not published"
                    ))
                });
                let outcome = import_result(entry, result);
                {
                    let mut state = progress
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    state.active.remove(&index);
                    state.completed += 1;
                    state.publish(entries.len(), &tx);
                }
                let _ = tx.send_blocking(BatchImportEvent::Result(outcome.clone()));
                (index, outcome)
            },
        );
        // Scoped workers, nod processors and staged-file cleanup have finished.
        // Emit an outcome for every unstarted item without opening its source.
        let mut delivered = vec![false; entries.len()];
        let mut summary = ImportSummary::default();
        for (index, outcome) in results {
            delivered[index] = true;
            summary.record(&outcome);
        }
        for (index, entry) in entries.into_iter().enumerate() {
            if !delivered[index] {
                let outcome = if cancellation.is_cancelled() {
                    ImportResult {
                        entry,
                        error: None,
                        warning: None,
                        cancelled: true,
                    }
                } else {
                    ImportResult {
                        entry,
                        error: Some("Import worker ended without a result".into()),
                        warning: None,
                        cancelled: false,
                    }
                };
                summary.record(&outcome);
                let _ = tx.send_blocking(BatchImportEvent::Result(outcome));
            }
        }
        let _ = tx.send_blocking(BatchImportEvent::Finished(summary));
    });
    rx
}

fn import_result(entry: &ImportEntry, result: Result<Option<String>>) -> ImportResult {
    let (error, warning, cancelled) = match result {
        Ok(warning) => (None, warning, false),
        Err(error) if is_cancel_error(&error) => {
            let warning = error
                .downcast_ref::<CleanupWarning>()
                .map(|_| format!("{error:#}"));
            (None, warning, true)
        }
        Err(error) => (Some(format!("{error:#}")), None, false),
    };
    ImportResult {
        entry: entry.clone(),
        error,
        warning,
        cancelled,
    }
}

impl ImportSummary {
    fn record(&mut self, outcome: &ImportResult) {
        if outcome.cancelled {
            self.cancelled += 1;
        } else if outcome.error.is_some() {
            self.failed += 1;
        } else {
            self.imported += 1;
        }
    }
}
pub fn get_filename(
    game_id: GameID,
    is_wii: bool,
    part: usize,
    disc_num: u8,
    wii_output_format: Format,
    gc_output_format: Format,
    must_split: bool,
) -> String {
    if is_wii {
        match wii_output_format {
            Format::Iso => {
                if must_split {
                    format!("{game_id}.part{part}.iso")
                } else {
                    format!("{game_id}.iso")
                }
            }
            _ => match part {
                0 => format!("{game_id}.wbfs"),
                n => format!("{game_id}.wbf{n}"),
            },
        }
    } else {
        match gc_output_format {
            Format::Ciso => match disc_num {
                0 => "game.ciso".to_string(),
                n => format!("disc{}.ciso", u16::from(n) + 1),
            },
            _ => match disc_num {
                0 => "game.iso".to_string(),
                n => format!("disc{}.iso", u16::from(n) + 1),
            },
        }
    }
}

fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || " !#$%&'()+,-.;=@^_`{}~".contains(c)
}

pub fn sanitize_title(ascii_title: &str) -> String {
    let mut title = ascii_title
        .chars()
        .skip_while(|c| !c.is_ascii_alphanumeric())
        .filter(|&c| is_valid_char(c))
        .take(64)
        .collect::<String>();

    // Remove trailing whitespace in-place
    title.truncate(title.trim_end().len());

    if title.is_empty() {
        title.push_str("game");
    }

    title
}

pub fn make_game_dir_name(game_id: GameID, fallback_title: &str) -> String {
    let game_title = {
        let ascii_title = twbm_idmap::get_ascii_title(game_id).unwrap_or(fallback_title);
        sanitize_title(ascii_title)
    };

    format!("{game_title} [{game_id}]")
}

fn wii_disc_info_to_nod_format(format: wii_disc_info::Format) -> Format {
    match format {
        wii_disc_info::Format::Wbfs => Format::Wbfs,
        wii_disc_info::Format::Ciso => Format::Ciso,
        wii_disc_info::Format::Iso => Format::Iso,
        wii_disc_info::Format::Rvz => Format::Rvz,
        wii_disc_info::Format::Wia => Format::Wia,
        wii_disc_info::Format::Gcz => Format::Gcz,
        wii_disc_info::Format::Tgc => Format::Tgc,
    }
}
