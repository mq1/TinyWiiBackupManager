use crate::game::{CalculatedHashes, Game, VerificationStatus};
use crate::messages::BackgroundMessage;
use crate::util::split::SplitWriter;
use anyhow::{Result, bail};
use log::info;
use nod::common::Format;
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions};
use sanitize_filename_reader_friendly::sanitize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fs, io};

/// Public entry point for the conversion process.
///
/// # Arguments
/// * `input_path` - Path to the source Wii or GameCube disc image.
/// * `output_dir` - Path to the directory where output files will be created.
/// * `sender` - Channel sender for progress updates.
/// * `cancelled` - Atomic flag to signal cancellation of the operation.
pub fn convert_game(
    input_path: &Path,
    output_dir: &Path,
    sender: egui_inbox::UiInboxSender<BackgroundMessage>,
    cancelled: Arc<AtomicBool>,
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
            // Check for cancellation
            if cancelled.load(Ordering::Relaxed) {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "Operation cancelled by user",
                ));
            }
            split_writer.write_all(data.as_ref())?;
            // Send progress updates
            let _ = sender.send(BackgroundMessage::OperationProgress(progress, total));
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

/// Verify a single game's integrity by processing the entire disc
pub fn verify_game(
    game: Box<Game>,
    sender: egui_inbox::UiInboxSender<BackgroundMessage>,
    cancelled: Arc<AtomicBool>,
) -> Result<VerificationStatus> {
    let disc_path = game
        .get_disc_file_path()
        .ok_or_else(|| anyhow::anyhow!("No disc image found"))?;

    // Open the disc
    let disc = DiscReader::new(
        &disc_path,
        &DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads: 1,
        },
    )?;
    let disc_writer = DiscWriter::new(disc, &FormatOptions::default())?;
    let total = disc_writer.progress_bound();

    // Process the disc to calculate hashes
    let finalization = disc_writer.process(
        |_data, pos, _| {
            // Check for cancellation
            if cancelled.load(Ordering::Relaxed) {
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "Operation cancelled by user",
                ));
            }
            // Send progress updates
            let _ = sender.send(BackgroundMessage::OperationProgress(pos, total));
            Ok(())
        },
        &ProcessOptions {
            processor_threads: 0,
            digest_crc32: true,
            digest_md5: false, // MD5 is slow, skip it
            digest_sha1: true,
            digest_xxh64: true,
        },
    )?;

    // Store calculated hashes
    let calculated = CalculatedHashes {
        crc32: finalization.crc32,
        sha1: finalization.sha1,
        xxh64: finalization.xxh64,
    };

    // Check against Redump database using the shared logic
    Ok(calculated.into_verification_status())
}
