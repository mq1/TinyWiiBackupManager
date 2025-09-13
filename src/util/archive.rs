// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::Game;
use crate::settings::ArchiveFormat;
use crate::util::concurrency::get_threads_num;
use crate::util::fs::find_disc;
use anyhow::{Context, Result};
use nod::common::{Compression, Format};
use nod::read::{DiscOptions, DiscReader, PartitionEncryption};
use nod::write::{DiscWriter, FormatOptions, ProcessOptions};
use sanitize_filename_reader_friendly::sanitize;
use std::fs::File;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};

pub fn game(
    game: &Game,
    output_dir: impl AsRef<Path>,
    archive_format: ArchiveFormat,
    mut progress_callback: impl FnMut(u64, u64),
) -> Result<PathBuf> {
    let input_path = find_disc(&game.path)?;

    let title = sanitize(&game.display_title);
    let output_path = output_dir
        .as_ref()
        .join(title)
        .with_extension(archive_format.extension());

    let (preloader_threads, processor_threads) = get_threads_num();

    let disc = DiscReader::new(
        &input_path,
        &DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads,
        },
    )
    .with_context(|| format!("Failed to read disc image: {}", input_path.display()))?;

    let mut output_file = File::create(&output_path)
        .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;

    let options = match archive_format {
        ArchiveFormat::Rvz => FormatOptions {
            format: Format::Rvz,
            compression: Compression::Zstandard(19),
            block_size: Format::Rvz.default_block_size(),
        },
        ArchiveFormat::Iso => FormatOptions {
            format: Format::Iso,
            compression: Compression::None,
            block_size: Format::Iso.default_block_size(),
        },
    };

    let writer = DiscWriter::new(disc, &options).context("Failed to initialize DiscWriter")?;

    let process_options = ProcessOptions {
        processor_threads,
        digest_crc32: true,
        digest_md5: false,
        digest_sha1: true,
        digest_xxh64: true,
        scrub_update_partition: false,
    };

    let finalization = writer
        .process(
            |data, progress, total| {
                output_file.write_all(data.as_ref())?;
                progress_callback(progress, total);
                Ok(())
            },
            &process_options,
        )
        .context("Failed to process disc for archival")?;

    if !finalization.header.is_empty() {
        output_file
            .rewind()
            .context("Failed to rewind output file")?;
        output_file
            .write_all(finalization.header.as_ref())
            .context("Failed to write final disc header")?;
    }
    output_file.flush().context("Failed to flush output file")?;

    Ok(output_path)
}
