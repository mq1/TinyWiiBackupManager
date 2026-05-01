// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::util::{BUF_SIZE, ext_to_format, format_to_opts, get_threads_num};
use anyhow::{Result, anyhow};
use nod::{
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, ProcessOptions, ScrubLevel},
};
use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
    path::Path,
};

pub fn perform(in_path: &Path, out_path: &Path, update_progress: &impl Fn(u8)) -> Result<()> {
    let out_ext = out_path.extension().ok_or(anyhow!("No extension"))?;
    let out_format = ext_to_format(out_ext).ok_or(anyhow!("Invalid extension"))?;

    let (processor_threads, preloader_threads) = get_threads_num();
    let disc_opts = DiscOptions {
        partition_encryption: PartitionEncryption::Original,
        preloader_threads,
    };

    let format_opts = format_to_opts(out_format);

    let process_opts = ProcessOptions {
        processor_threads,
        scrub: ScrubLevel::None,
        digest_crc32: true,
        digest_md5: false,
        digest_sha1: true,
        digest_xxh64: true,
    };

    let disc_reader = DiscReader::new(in_path, &disc_opts)?;
    let disc_writer = DiscWriter::new(disc_reader, &format_opts)?;

    let mut out_writer = BufWriter::with_capacity(BUF_SIZE, File::create(out_path)?);
    let mut last_percentage = 0;

    let finalization = disc_writer.process(
        |data, progress, size| {
            out_writer.write_all(&data)?;

            let current_percentage = (progress * 100 / size) as u8;

            if current_percentage != last_percentage {
                update_progress(current_percentage);
                last_percentage = current_percentage;
            }

            Ok(())
        },
        &process_opts,
    )?;

    let mut out_file = out_writer
        .into_inner()
        .map_err(|_| anyhow!("Failed to get inner split writer"))?;

    if !finalization.header.is_empty() {
        out_file.rewind()?;
        out_file.write_all(&finalization.header)?;
    }

    out_file.flush()?;

    Ok(())
}
