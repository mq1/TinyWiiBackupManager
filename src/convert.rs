// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    TaskType, WiiOutputFormat,
    config::Config,
    extensions::{SUPPORTED_INPUT_EXTENSIONS, get_convert_extension},
    tasks::TaskProcessor,
    util,
};
use anyhow::{Result, anyhow, bail};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions},
};
use rfd::FileDialog;
use slint::ToSharedString;
use std::{
    fs::{self, File},
    io::{BufWriter, Seek, Write},
    sync::{Arc, Mutex},
};

fn get_disc_opts() -> DiscOptions {
    let (preloader_threads, _) = util::get_threads_num();

    DiscOptions {
        partition_encryption: PartitionEncryption::Original,
        preloader_threads,
    }
}

fn get_process_opts(config: &Config) -> ProcessOptions {
    let scrub_update_partition = config.scrub_update_partition;
    let (_, processor_threads) = util::get_threads_num();

    ProcessOptions {
        processor_threads,
        digest_crc32: true,
        digest_md5: false, // too slow
        digest_sha1: true,
        digest_xxh64: true,
        scrub_update_partition,
    }
}

fn get_output_format_opts(config: &Config) -> FormatOptions {
    match config.wii_output_format {
        WiiOutputFormat::WbfsAuto | WiiOutputFormat::WbfsFixed => FormatOptions::new(Format::Wbfs),
        WiiOutputFormat::Iso => FormatOptions::new(Format::Iso),
    }
}

pub fn add_games(config: &Arc<Mutex<Config>>, task_processor: &Arc<TaskProcessor>) -> Result<()> {
    let config = config.lock().map_err(|_| anyhow!("Mutex poisoned"))?;

    let mount_point = &config.mount_point;
    if mount_point.as_os_str().is_empty() {
        bail!("No mount point selected");
    }

    let paths = FileDialog::new()
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .pick_files()
        .ok_or(anyhow!("No Games Selected"))?;

    for path in paths {
        let disc_opts = get_disc_opts();
        let process_opts = get_process_opts(&config);
        let mount_point = mount_point.clone();
        let config = config.clone();

        task_processor.spawn(Box::new(move |weak| {
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
                .with_extension(get_convert_extension(&config, is_wii));

            let mut out = BufWriter::new(File::create(&path)?);

            let out_opts = get_output_format_opts(&config);
            let writer = DiscWriter::new(disc, &out_opts)?;
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
        }))?;
    }

    Ok(())
}
