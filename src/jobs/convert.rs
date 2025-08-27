use super::{
    Job, JobContext, JobResult, JobState, should_cancel, start_job, update_status_with_bytes,
};
use crate::game::CalculatedHashes;
use crate::util::split::SplitWriter;
use anyhow::{Result, bail};
use log::info;
use nod::common::Format;
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions};
use sanitize_filename_reader_friendly::sanitize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::task::Waker;

pub struct ConvertConfig {
    pub base_dir: PathBuf,
    pub paths: Vec<PathBuf>,
    pub remove_sources: bool,
}

pub struct ConvertResult {
    pub converted: usize,
    pub failed: usize,
    pub failed_paths: Vec<PathBuf>,
}

pub fn start_convert(waker: Waker, config: ConvertConfig) -> JobState {
    let title = format!(
        "Convert {} file{}",
        config.paths.len(),
        if config.paths.len() == 1 { "" } else { "s" }
    );

    start_job(waker, &title, Job::Convert, move |context, cancel| {
        convert_games(context, cancel, config).map(JobResult::Convert)
    })
}

fn convert_games(
    context: JobContext,
    cancel: Receiver<()>,
    config: ConvertConfig,
) -> Result<Box<ConvertResult>> {
    let mut converted = 0;
    let mut failed = 0;
    let mut failed_paths = Vec::new();
    let mut was_cancelled = false;

    let total = config.paths.len() as u32;

    for (idx, path) in config.paths.iter().enumerate() {
        // Check for cancellation
        if should_cancel(&cancel) {
            was_cancelled = true;
            break;
        }

        // Update status with current file
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Perform the conversion
        match convert_game(
            path,
            &config.base_dir,
            idx as u32,
            total,
            file_name.clone(),
            &context,
            &cancel,
        ) {
            Ok((game_path, _calculated_hashes)) => {
                converted += 1;
                log::info!("Converted {} to {:?}", file_name, game_path);

                // Remove source if requested
                if config.remove_sources {
                    if let Err(e) = fs::remove_file(path) {
                        log::warn!("Failed to remove source file {}: {}", path.display(), e);
                    }
                }
            }
            Err(e) => {
                // Check if it was cancelled
                if e.to_string().contains("Cancelled") {
                    was_cancelled = true;
                    break;
                }
                failed += 1;
                failed_paths.push(path.clone());
                log::warn!("Failed to convert {}: {}", path.display(), e);
            }
        }
    }

    // Final status
    let final_status = if was_cancelled {
        format!("Conversion cancelled ({} completed)", converted)
    } else {
        format!(
            "Converted {} file{}",
            converted,
            if converted == 1 { "" } else { "s" }
        )
    };

    let _ = update_status_with_bytes(&context, final_status, total, total, 1, 1, None, &cancel);

    Ok(Box::new(ConvertResult {
        converted,
        failed,
        failed_paths,
    }))
}

/// Convert a single game disc image to WBFS/CISO format
fn convert_game(
    input_path: &Path,
    output_dir: &Path,
    current_idx: u32,
    total_items: u32,
    file_name: String,
    context: &JobContext,
    cancel: &Receiver<()>,
) -> Result<(PathBuf, CalculatedHashes)> {
    info!("Opening disc image: {}", input_path.display());

    // Set the number of threads based on the number of available CPUs.
    let cpus = num_cpus::get();
    let preloader_threads = if cpus <= 4 {
        1
    } else if cpus <= 8 {
        2
    } else {
        4
    };
    let processor_threads = (cpus - preloader_threads).max(1);

    let disc = DiscReader::new(
        input_path,
        &DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads,
        },
    )?;

    let header = disc.header();
    let game_id = header.game_id_str();
    let game_title = header.game_title_str();
    let sanitized_title = sanitize(game_title);
    let game_dir_name = format!("{} [{}]", sanitized_title, game_id);

    // Determine output format and path based on disc type
    let (format, game_output_dir, output_file_path) = if header.is_wii() {
        info!("Detected Wii disc. Converting to split WBFS format.");
        let game_output_dir = output_dir.join("wbfs").join(&game_dir_name);
        fs::create_dir_all(&game_output_dir)?;
        let base_path = game_output_dir.join(format!("{}.wbfs", game_id));
        (Format::Wbfs, game_output_dir, base_path)
    } else if header.is_gamecube() {
        info!("Detected GameCube disc. Converting to CISO format.");
        let game_output_dir = output_dir.join("games").join(&game_dir_name);
        fs::create_dir_all(&game_output_dir)?;
        // Nintendont naming convention
        let iso_filename = match header.disc_num {
            0 => "game.iso".to_string(),
            n => format!("disc{}.iso", n + 1),
        };
        let base_path = game_output_dir.join(iso_filename);
        (Format::Ciso, game_output_dir, base_path)
    } else {
        bail!("Input file is not a valid Wii or GameCube disc.");
    };

    let mut split_writer = SplitWriter::new(&output_file_path);
    let format_options = FormatOptions::new(format);
    let disc_writer = DiscWriter::new(disc, &format_options)?;

    let process_options = ProcessOptions {
        processor_threads,
        digest_crc32: true,
        digest_md5: false, // MD5 is slow, skip it
        digest_sha1: true,
        digest_xxh64: true,
    };

    info!("Processing disc with {} threads", processor_threads);

    let finalization = disc_writer.process(
        |data, progress, total| {
            // Check for cancellation via should_cancel check in update_status_with_bytes
            split_writer.write_all(data.as_ref())?;

            // Send progress updates (this will check for cancellation)
            update_status_with_bytes(
                context,
                format!("Converting {}", file_name),
                current_idx,
                total_items,
                progress,
                total,
                Some(file_name.clone()),
                cancel,
            )
            .map_err(|_| io::Error::new(io::ErrorKind::Interrupted, "Cancelled"))?;

            Ok(())
        },
        &process_options,
    )?;

    // Write final header if needed (required for WBFS and CISO formats)
    if !finalization.header.is_empty() {
        split_writer.write_all_at(0, finalization.header.as_ref())?;
    }

    split_writer.finalize()?;

    // Store calculated hashes from conversion
    let calculated = CalculatedHashes {
        crc32: finalization.crc32,
        sha1: finalization.sha1,
        xxh64: finalization.xxh64,
    };

    info!("Conversion complete!");
    Ok((game_output_dir, calculated))
}
