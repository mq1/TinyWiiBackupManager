// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::WiiOutputFormat,
    overflow_reader::{OverflowReader, get_overflow_file},
    tasks::BackgroundMessage,
    util::{self, can_write_over_4gb},
};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use sanitize_filename::sanitize;
use std::{
    fs::{self, File},
    io::{BufWriter, Seek, Write},
    path::PathBuf,
};

const SPLIT_SIZE: u64 = 4 * 1024 * 1024 * 1024 - 32 * 1024;

pub fn get_disc_opts() -> DiscOptions {
    let (preloader_threads, _) = util::get_threads_num();

    DiscOptions {
        partition_encryption: PartitionEncryption::Original,
        preloader_threads,
    }
}

pub fn get_process_opts(scrub_update_partition: bool) -> ProcessOptions {
    let (_, processor_threads) = util::get_threads_num();

    let scrub = match scrub_update_partition {
        true => ScrubLevel::UpdatePartition,
        false => ScrubLevel::None,
    };

    ProcessOptions {
        processor_threads,
        digest_crc32: true,
        digest_md5: false, // too slow
        digest_sha1: true,
        digest_xxh64: true,
        scrub,
    }
}

fn get_output_format_opts(wii_output_format: WiiOutputFormat, is_wii: bool) -> FormatOptions {
    match (wii_output_format, is_wii) {
        (WiiOutputFormat::Wbfs, true) => FormatOptions::new(Format::Wbfs),
        (WiiOutputFormat::Iso, true) => FormatOptions::new(Format::Iso),
        (_, false) => FormatOptions::new(Format::Ciso),
    }
}

pub fn spawn_add_games_task(app: &App, mut paths: Vec<PathBuf>) {
    let remove_sources = app.config.contents.remove_sources_games;
    let mount_point_clone = app.config.contents.mount_point.clone();
    let wii_output_format = app.config.contents.wii_output_format;
    let disc_opts = get_disc_opts();
    let scrub_update_partition = app.config.contents.scrub_update_partition;
    let must_split =
        app.config.contents.always_split || can_write_over_4gb(&mount_point_clone).is_err();

    // We'll get those later with get_overflow_file
    paths.retain(|path| !path.ends_with(".part1.iso"));

    app.task_processor.spawn(move |msg_sender| {
        let len = paths.len();
        for (i, path) in paths.into_iter().enumerate() {
            let (title, id, is_wii, disc_num) = {
                let reader = DiscReader::new(&path, &disc_opts)?;
                let header = reader.header();
                let title = header.game_title_str().to_string();
                let id = header.game_id_str().to_string();
                let is_wii = header.is_wii();
                let disc_num = header.disc_num;
                (title, id, is_wii, disc_num)
            };

            let dir_path = mount_point_clone
                .join(if is_wii { "wbfs" } else { "games" })
                .join(format!("{} [{}]", sanitize(&title), id));

            let file_name1 = match (is_wii, wii_output_format, must_split, disc_num) {
                (true, WiiOutputFormat::Wbfs, _, _) => &format!("{id}.wbfs"),
                (true, WiiOutputFormat::Iso, true, _) => &format!("{id}.part0.iso"),
                (true, WiiOutputFormat::Iso, false, _) => &format!("{id}.iso"),
                (false, _, _, 0) => "game.iso",
                (false, _, _, n) => &format!("disc{n}.iso"),
            };

            let path1 = dir_path.join(file_name1);

            if path1.exists() {
                continue;
            }

            fs::create_dir_all(&dir_path)?;

            {
                let overflow_file = get_overflow_file(&path);
                let disc = if let Some(overflow_file) = overflow_file {
                    let reader = OverflowReader::new(&path, &overflow_file)?;
                    DiscReader::new_stream(Box::new(reader), &disc_opts)?
                } else {
                    DiscReader::new(&path, &disc_opts)?
                };

                let mut out1 = BufWriter::new(File::create(&path1)?);

                let file_name2 = match wii_output_format {
                    WiiOutputFormat::Wbfs => &format!("{id}.wbf1"),
                    WiiOutputFormat::Iso => &format!("{id}.part1.iso"),
                };
                let path2 = dir_path.join(file_name2);
                let mut out2: Option<BufWriter<File>> = None;

                let out_opts = get_output_format_opts(wii_output_format, is_wii);
                let writer = DiscWriter::new(disc, &out_opts)?;

                let process_opts = get_process_opts(
                    scrub_update_partition && is_wii && wii_output_format == WiiOutputFormat::Wbfs,
                );

                let finalization = writer.process(
                    |data, progress, total| {
                        // get position
                        let pos = out1.stream_position()?;

                        // write data to out1, or overflow to out2
                        if let Some(out2) = out2.as_mut() {
                            out2.write_all(&data)?;
                        } else if is_wii && must_split && pos + data.len() as u64 > SPLIT_SIZE {
                            let mut writer = BufWriter::new(File::create(&path2)?);
                            writer.write_all(&data)?;
                            out2 = Some(writer);
                        } else {
                            out1.write_all(&data)?;
                        }

                        msg_sender
                            .send(BackgroundMessage::UpdateStatus(format!(
                                "🎮 Adding {}  {:02.0}%  ({}/{})",
                                title,
                                progress as f32 / total as f32 * 100.0,
                                i + 1,
                                len
                            )))
                            .map_err(|e| std::io::Error::other(e.to_string()))?;

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

            msg_sender.send(BackgroundMessage::NotifyInfo(format!("🎮 Added {}", title)))?;
            msg_sender.send(BackgroundMessage::TriggerRefreshGames)?;
        }

        Ok(())
    });
}
