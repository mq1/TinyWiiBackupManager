// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs::{self, File},
    io::{BufWriter, Seek, Write},
};

use crate::{concurrency::get_threads_num, config};
use anyhow::{Result, anyhow, bail};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use rfd::FileDialog;

pub fn add_games() -> Result<()> {
    let config = config::get();

    let mount_point = config.mount_point;
    if mount_point.as_os_str().is_empty() {
        bail!("No mount point selected");
    }

    let paths = FileDialog::new()
        .add_filter("Nintendo Optical Disc", &["iso", "rvz"])
        .pick_files()
        .ok_or(anyhow!("No Games Selected"))?;

    let (preloader_threads, processor_threads) = get_threads_num();

    let disc_opts = DiscOptions {
        partition_encryption: PartitionEncryption::Original,
        preloader_threads,
    };

    let process_opts = ProcessOptions {
        processor_threads,
        digest_crc32: true,
        digest_md5: false,
        digest_sha1: true,
        digest_xxh64: true,
        scrub_update_partition: config.scrub_update_partition,
    };

    for path in paths {
        let disc = DiscReader::new(path, &disc_opts)?;
        let header = disc.header().clone();

        let dir_path = mount_point
            .join(if header.is_wii() { "wbfs" } else { "games" })
            .join(format!(
                "{} [{}]",
                header.game_title_str(),
                header.game_id_str()
            ));

        if dir_path.exists() {
            continue;
        }

        fs::create_dir_all(&dir_path)?;

        let path = dir_path
            .join(header.game_id_str())
            .with_extension(if header.is_wii() { "wbfs" } else { "iso" });

        let mut out = BufWriter::new(File::create(&path)?);

        let writer = DiscWriter::new(disc, &FormatOptions::new(Format::Wbfs))?;
        let finalization = writer.process(
            |data, _progress, _total| {
                out.write_all(&data)?;
                Ok(())
            },
            &process_opts,
        )?;

        if !finalization.header.is_empty() {
            out.rewind()?;
            out.write_all(&finalization.header)?;
        }
    }

    Ok(())
}
