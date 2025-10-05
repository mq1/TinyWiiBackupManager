// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    fs::{self, File},
    io::{BufWriter, Seek, Write},
};

use crate::{TaskType, concurrency::get_threads_num, config, tasks};
use anyhow::{Result, anyhow, bail};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use rfd::FileDialog;
use slint::ToSharedString;

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
        let disc_opts = disc_opts.clone();
        let process_opts = process_opts.clone();
        let mount_point = mount_point.clone();

        tasks::spawn_task(Box::new(move |weak| {
            let _ = weak.upgrade_in_event_loop(move |handle| {
                handle.set_task_type(TaskType::Converting);
            });

            let disc = DiscReader::new(path, &disc_opts)?;

            let header = disc.header().clone();
            let title = header.game_title_str();
            let id = header.game_id_str();
            let is_wii = header.is_wii();

            let dir_path = mount_point
                .join(if is_wii { "wbfs" } else { "games" })
                .join(format!("{title} [{id}]"));

            if dir_path.exists() {
                return Ok(());
            }

            fs::create_dir_all(&dir_path)?;

            let path = dir_path
                .join(id)
                .with_extension(if is_wii { "wbfs" } else { "iso" });

            let mut out = BufWriter::new(File::create(&path)?);

            let writer = DiscWriter::new(disc, &FormatOptions::new(Format::Wbfs))?;
            let finalization = writer.process(
                |data, progress, total| {
                    out.write_all(&data)?;

                    let status = format!(
                        "Adding {}  {:02.0}%",
                        title,
                        progress as f32 / total as f32 * 100.0,
                    );

                    let _ = weak.upgrade_in_event_loop(move |handle| {
                        handle.set_status(status.to_shared_string());
                    });

                    Ok(())
                },
                &process_opts,
            )?;

            if !finalization.header.is_empty() {
                out.rewind()?;
                out.write_all(&finalization.header)?;
            }

            Ok(())
        }));
    }

    Ok(())
}
