// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    Config, TaskType, WiiOutputFormat,
    covers::download_covers,
    extensions::SUPPORTED_INPUT_EXTENSIONS,
    tasks::TaskProcessor,
    util::{self, can_write_over_4gb},
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
    path::{Path, PathBuf},
    sync::Arc,
};

const SPLIT_SIZE: u64 = 4 * 1024 * 1024 * 1024 - 32 * 1024;

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
        WiiOutputFormat::Wbfs => FormatOptions::new(Format::Wbfs),
        WiiOutputFormat::Iso => FormatOptions::new(Format::Iso),
    }
}

pub fn add_games(
    data_dir: &Path,
    task_processor: &Arc<TaskProcessor>,
    remove_sources: bool,
) -> Result<()> {
    let config = Config::load(data_dir);
    if config.mount_point.is_empty() {
        bail!("Conversion Failed: No mount point selected");
    }

    let mount_point = PathBuf::from(&config.mount_point);
    let wii_output_format = config.wii_output_format;
    let disc_opts = get_disc_opts();
    let process_opts = get_process_opts(&config);
    let out_opts = get_output_format_opts(&config);
    let must_split = config.always_split || can_write_over_4gb(&mount_point).is_err();

    let paths = FileDialog::new()
        .add_filter("Nintendo Optical Disc", SUPPORTED_INPUT_EXTENSIONS)
        .pick_files()
        .ok_or(anyhow!("No Games Selected"))?;

    let mount_point_clone = mount_point.clone();
    task_processor.spawn(Box::new(move |weak| {
        weak.upgrade_in_event_loop(|handle| {
            handle.set_task_type(TaskType::Converting);
        })?;

        let len = paths.len();
        for (i, path) in paths.into_iter().enumerate() {
            {
                let disc = DiscReader::new(&path, &disc_opts)?;

                let header = disc.header().clone();
                let title = header.game_title_str().to_string();
                let id = header.game_id_str();
                let is_wii = header.is_wii();

                let dir_path = mount_point_clone
                    .join(if is_wii { "wbfs" } else { "games" })
                    .join(format!("{title} [{id}]"));

                if dir_path.exists() {
                    return Ok(());
                }

                fs::create_dir_all(&dir_path)?;

                let base_path = dir_path.join(id);

                let path1 = match (wii_output_format, must_split) {
                    (WiiOutputFormat::Wbfs, _) => base_path.with_extension("wbfs"),
                    (WiiOutputFormat::Iso, true) => base_path.with_extension("part0.iso"),
                    (WiiOutputFormat::Iso, false) => base_path.with_extension("iso"),
                };

                let mut out1 = BufWriter::new(File::create(&path1)?);

                let path2 = match wii_output_format {
                    WiiOutputFormat::Wbfs => base_path.with_extension("wbf1"),
                    WiiOutputFormat::Iso => base_path.with_extension("part1.iso"),
                };

                let mut out2 = None;

                let writer = DiscWriter::new(disc, &out_opts)?;
                let finalization = writer.process(
                    |data, progress, total| {
                        // get position
                        let pos = out1.stream_position()?;

                        // write data to out1, or overflow to out2
                        if must_split && pos > SPLIT_SIZE {
                            out2.get_or_insert(BufWriter::new(File::create(&path2)?))
                                .write_all(&data)?;
                        } else {
                            out1.write_all(&data)?;
                        }

                        let status = format!(
                            "Adding {}  {:02.0}%  ({}/{})",
                            title,
                            progress as f32 / total as f32 * 100.0,
                            i + 1,
                            len
                        );

                        let _ = weak.upgrade_in_event_loop(move |handle| {
                            handle.set_status(status.to_shared_string());
                        });

                        Ok(())
                    },
                    &process_opts,
                )?;

                if !finalization.header.is_empty() {
                    out1.rewind()?;
                    out1.write_all(&finalization.header)?;
                }
            }

            if remove_sources {
                fs::remove_file(&path)?;
            }
        }

        Ok(())
    }))?;

    // Download covers (ignores errors)
    let mount_point_clone = mount_point.clone();
    task_processor.spawn(Box::new(move |weak| {
        weak.upgrade_in_event_loop(|handle| {
            handle.set_task_type(TaskType::DownloadingCovers);
            handle.set_status("Downloading Missing Covers...".to_shared_string());
        })?;

        download_covers(&mount_point_clone)
    }))?;

    Ok(())
}
