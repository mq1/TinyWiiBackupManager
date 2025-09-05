// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

//! A Rust library to convert Wii and GameCube disc images, replicating the
//! default behavior of `wbfs_file v2.9` for Wii and creating NKit-compatible
//! scrubbed ISOs for GameCube.

use crate::settings::WiiOutputFormat;
use crate::util::concurrency::get_threads_num;
use crate::util::fs::can_write_over_4gb;
use anyhow::{Context, Result, bail};
use nod::common::Format;
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions};
use sanitize_filename_reader_friendly::sanitize;
use std::fs::{self, File};
use std::io::{Seek, Write};
use std::path::Path;
use crate::util::split::SplitWbfsFile;

/// The fixed split size for output files: 4 GiB - 32 KiB.
const FIXED_SPLIT_SIZE: usize = (4 * 1024 * 1024 * 1024) - (32 * 1024);
const NO_SPLIT_SIZE: usize = usize::MAX;

/// Returns true if the game is already present in the output directory.
pub fn convert(
    input_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    wii_output_format: WiiOutputFormat,
    mut progress_callback: impl FnMut(u64, u64),
) -> Result<bool> {
    let input_path = input_path.as_ref();
    let output_dir = output_dir.as_ref();

    let (preloader_threads, processor_threads) = get_threads_num();

    let mut disc = DiscReader::new(
        &input_path,
        &DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads,
        },
    )
    .with_context(|| format!("Failed to read disc image: {}", input_path.display()))?;

    let header = disc.header();
    let game_id = header.game_id_str();
    let game_title = header.game_title_str();
    let sanitized_title = sanitize(game_title);
    let game_dir_name = format!("{sanitized_title} [{game_id}]");

    let game_output_dir = if header.is_wii() {
        output_dir.join("wbfs").join(&game_dir_name)
    } else if header.is_gamecube() {
        output_dir.join("games").join(&game_dir_name)
    } else {
        bail!("Invalid disc");
    };

    // If the game is already present, return true.
    if game_output_dir.exists() {
        return Ok(true);
    }

    fs::create_dir_all(&game_output_dir).with_context(|| {
        format!("Failed to create directory: {}", game_output_dir.display())
    })?;

    if header.is_wii() {
        let split_size = match (can_write_over_4gb(output_dir), wii_output_format) {
            (true, WiiOutputFormat::WbfsAuto | WiiOutputFormat::Iso) => NO_SPLIT_SIZE,
            (_, WiiOutputFormat::WbfsFixed | WiiOutputFormat::WbfsAuto) => FIXED_SPLIT_SIZE,
            (false, WiiOutputFormat::Iso) => bail!("Can't create ISO file on this platform"),
        };

        let base_path = game_output_dir.join(game_id);

        if matches!(
            wii_output_format,
            WiiOutputFormat::WbfsAuto | WiiOutputFormat::WbfsFixed
        ) {
            let mut writer = SplitWbfsFile::new(&base_path, split_size)?;
            let format_options = FormatOptions::new(Format::Wbfs);
            let disc_writer = DiscWriter::new(disc, &format_options)
                .context("Failed to initialize WBFS writer")?;

            let process_options = ProcessOptions {
                processor_threads,
                digest_crc32: true,
                digest_md5: false,
                digest_sha1: true,
                digest_xxh64: true,
            };

            let finalization = disc_writer
                .process(
                    |data, progress, total| {
                        writer.write(&data)?;
                        progress_callback(progress, total);
                        Ok(())
                    },
                    &process_options,
                )
                .context("Failed to process disc for WBFS conversion")?;

            if !finalization.header.is_empty() {
                writer.write_to_start(&finalization.header).context("Failed to write header")?;
            }
        } else {
            let out_path = base_path.with_extension("iso");
            let mut out_file = File::create(&out_path).context("Failed to create output file")?;
            nod::util::buf_copy(&mut disc, &mut out_file).context("Failed to copy data")?;
        }
    } else if header.is_gamecube() {
        let iso_filename = match header.disc_num {
            0 => "game.iso".to_string(),
            n => format!("disc{}.iso", n + 1),
        };
        let output_iso_path = game_output_dir.join(iso_filename);

        let mut out_file = File::create(&output_iso_path).with_context(|| {
            format!(
                "Failed to create output file: {}",
                output_iso_path.display()
            )
        })?;

        let format_options = FormatOptions::new(Format::Ciso);
        let disc_writer =
            DiscWriter::new(disc, &format_options).context("Failed to initialize CISO writer")?;

        let process_options = ProcessOptions {
            processor_threads,
            digest_crc32: true,
            digest_md5: false,
            digest_sha1: true,
            digest_xxh64: true,
        };

        let finalization = disc_writer
            .process(
                |data, progress, total| {
                    out_file.write_all(&data)?;
                    progress_callback(progress, total);
                    Ok(())
                },
                &process_options,
            )
            .context("Failed to process disc for CISO conversion")?;

        if !finalization.header.is_empty() {
            out_file.rewind().context("Failed to rewind output file")?;
            out_file
                .write_all(&finalization.header)
                .context("Failed to write final CISO header")?;
        }
    }

    Ok(false)
}
