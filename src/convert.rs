// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::messages::Message;
use crate::{
    config::{GcOutputFormat, WiiOutputFormat},
    disc_info::DiscInfo,
    overflow_reader::{OverflowReader, get_overflow_file},
    overflow_writer::OverflowWriter,
    util::{self, can_write_over_4gb},
};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use sanitize_filename::sanitize;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::{
    fs::{self},
    io,
    io::Write,
    time::Instant,
};
use tempfile::tempfile;
use zip::ZipArchive;

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
        digest_md5: false,
        digest_sha1: false,
        digest_xxh64: false,
        scrub,
    }
}

fn get_output_format_opts(
    wii_output_format: WiiOutputFormat,
    gc_output_format: GcOutputFormat,
    is_wii: bool,
) -> FormatOptions {
    match (wii_output_format, gc_output_format, is_wii) {
        (WiiOutputFormat::Wbfs, _, true) => FormatOptions::new(Format::Wbfs),
        (WiiOutputFormat::Iso, _, true) => FormatOptions::new(Format::Iso),
        (_, GcOutputFormat::Iso, false) => FormatOptions::new(Format::Iso),
        (_, GcOutputFormat::Ciso, false) => FormatOptions::new(Format::Ciso),
    }
}

pub fn spawn_add_games_task(app: &App, discs: Box<[DiscInfo]>) {
    let disc_opts = get_disc_opts();

    let mount_point = app.config.contents.mount_point.clone();
    let wii_output_format = app.config.contents.wii_output_format;
    let gc_output_format = app.config.contents.gc_output_format;
    let scrub_update_partition = app.config.contents.scrub_update_partition;
    let remove_sources_games = app.config.contents.remove_sources_games;

    let must_split = app.config.contents.always_split
        || can_write_over_4gb(&app.config.contents.mount_point).is_err();

    app.task_processor.spawn(move |msg_sender| {
        let len = discs.len();
        for (i, disc_info) in discs.into_iter().enumerate() {
            let dir_path = mount_point
                .join(if disc_info.header.is_wii() {
                    "wbfs"
                } else {
                    "games"
                })
                .join(format!(
                    "{} [{}]",
                    sanitize(disc_info.header.game_title_str()),
                    disc_info.header.game_id_str()
                ));

            let file_name1 = match (
                disc_info.header.is_wii(),
                wii_output_format,
                gc_output_format,
                must_split,
                disc_info.header.disc_num,
            ) {
                (true, WiiOutputFormat::Wbfs, _, _, _) => {
                    &format!("{}.wbfs", disc_info.header.game_id_str())
                }
                (true, WiiOutputFormat::Iso, _, true, _) => {
                    &format!("{}.part0.iso", disc_info.header.game_id_str())
                }
                (true, WiiOutputFormat::Iso, _, false, _) => {
                    &format!("{}.iso", disc_info.header.game_id_str())
                }
                (false, _, GcOutputFormat::Iso, _, 0) => "game.iso",
                (false, _, GcOutputFormat::Iso, _, n) => &format!("disc{}.iso", n + 1),
                (false, _, GcOutputFormat::Ciso, _, 0) => "game.ciso",
                (false, _, GcOutputFormat::Ciso, _, n) => &format!("disc{}.ciso", n + 1),
            };

            let overflow_file = get_overflow_file(&disc_info.main_disc_path);

            let file_path1 = dir_path.join(file_name1);

            let file_name2 = match wii_output_format {
                WiiOutputFormat::Wbfs => &format!("{}.wbf1", disc_info.header.game_id_str()),
                WiiOutputFormat::Iso => &format!("{}.part1.iso", disc_info.header.game_id_str()),
            };
            let file_path2 = dir_path.join(file_name2);

            fs::create_dir_all(&dir_path)?;

            let start_instant = Instant::now();
            let mut tmp = tempfile()?;
            log::info!("Converting {}", disc_info.header.game_title_str());
            {
                let disc = if let Some(overflow_file) = &overflow_file {
                    let reader = OverflowReader::new(&disc_info.main_disc_path, overflow_file)?;
                    DiscReader::new_from_non_cloneable_read(reader, &disc_opts)?
                } else if disc_info
                    .main_disc_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ["zip", "ZIP"].contains(&ext))
                {
                    {
                        let file_reader = BufReader::new(File::open(&disc_info.main_disc_path)?);
                        let mut archive = ZipArchive::new(file_reader)?;
                        let mut disc_file = archive.by_index(0)?;
                        let mut tmp_writer = BufWriter::new(&mut tmp);

                        msg_sender.send(Message::UpdateStatus(format!(
                            "🎮 Extracting {}",
                            disc_info.header.game_title_str()
                        )))?;
                        io::copy(&mut disc_file, &mut tmp_writer)?;
                    }

                    DiscReader::new_from_non_cloneable_read(tmp, &disc_opts)?
                } else {
                    DiscReader::new(&disc_info.main_disc_path, &disc_opts)?
                };

                let mut overflow_writer = OverflowWriter::new(&file_path1, file_path2, must_split)?;

                let out_opts = get_output_format_opts(
                    wii_output_format,
                    gc_output_format,
                    disc_info.header.is_wii(),
                );
                let writer = DiscWriter::new(disc, &out_opts)?;

                let process_opts = get_process_opts(
                    scrub_update_partition
                        && disc_info.header.is_wii()
                        && wii_output_format == WiiOutputFormat::Wbfs,
                );

                let finalization = writer.process(
                    |data, progress, total| {
                        overflow_writer.write_all(&data)?;

                        let _ = msg_sender.send(Message::UpdateStatus(format!(
                            "🎮 Adding {}  {:02.0}%  ({}/{})",
                            disc_info.header.game_title_str(),
                            progress as f32 / total as f32 * 100.0,
                            i + 1,
                            len
                        )));

                        Ok(())
                    },
                    &process_opts,
                )?;

                if !finalization.header.is_empty() {
                    overflow_writer.write_header(&finalization.header)?;
                }
            }
            log::info!(
                "Converted {} in {:.2}s",
                disc_info.header.game_title_str(),
                start_instant.elapsed().as_secs_f32()
            );

            if remove_sources_games {
                fs::remove_file(&disc_info.main_disc_path)?;
                if let Some(overflow_file) = &overflow_file {
                    fs::remove_file(overflow_file)?;
                }
            }

            msg_sender.send(Message::NotifyInfo(format!(
                "🎮 Added {}",
                disc_info.header.game_title_str()
            )))?;
            msg_sender.send(Message::TriggerRefreshGames)?;
        }

        Ok(())
    });
}

pub fn spawn_convert_game_task(app: &App, disc_info: DiscInfo, dest_dir: PathBuf) {
    let disc_opts = get_disc_opts();

    let wii_output_format = app.config.contents.wii_output_format;
    let gc_output_format = app.config.contents.gc_output_format;
    let scrub_update_partition = app.config.contents.scrub_update_partition;
    let remove_sources_games = app.config.contents.remove_sources_games;

    let must_split = app.config.contents.always_split
        || can_write_over_4gb(&app.config.contents.mount_point).is_err();

    app.task_processor.spawn(move |msg_sender| {
        let dir_path = dest_dir.join(format!(
            "{} [{}]",
            sanitize(disc_info.header.game_title_str()),
            disc_info.header.game_id_str()
        ));

        let file_name1 = match (
            disc_info.header.is_wii(),
            wii_output_format,
            gc_output_format,
            must_split,
            disc_info.header.disc_num,
        ) {
            (true, WiiOutputFormat::Wbfs, _, _, _) => {
                &format!("{}.wbfs", disc_info.header.game_id_str())
            }
            (true, WiiOutputFormat::Iso, _, true, _) => {
                &format!("{}.part0.iso", disc_info.header.game_id_str())
            }
            (true, WiiOutputFormat::Iso, _, false, _) => {
                &format!("{}.iso", disc_info.header.game_id_str())
            }
            (false, _, GcOutputFormat::Iso, _, 0) => "game.iso",
            (false, _, GcOutputFormat::Iso, _, n) => &format!("disc{}.iso", n + 1),
            (false, _, GcOutputFormat::Ciso, _, 0) => "game.ciso",
            (false, _, GcOutputFormat::Ciso, _, n) => &format!("disc{}.ciso", n + 1),
        };

        let overflow_file = get_overflow_file(&disc_info.main_disc_path);

        let file_path1 = dir_path.join(file_name1);

        let file_name2 = match wii_output_format {
            WiiOutputFormat::Wbfs => &format!("{}.wbf1", disc_info.header.game_id_str()),
            WiiOutputFormat::Iso => &format!("{}.part1.iso", disc_info.header.game_id_str()),
        };
        let file_path2 = dir_path.join(file_name2);

        fs::create_dir_all(&dir_path)?;

        let mut tmp = tempfile()?;
        {
            let disc = if let Some(overflow_file) = &overflow_file {
                let reader = OverflowReader::new(&disc_info.main_disc_path, overflow_file)?;
                DiscReader::new_from_non_cloneable_read(reader, &disc_opts)?
            } else if disc_info
                .main_disc_path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ["zip", "ZIP"].contains(&ext))
            {
                {
                    let file_reader = BufReader::new(File::open(&disc_info.main_disc_path)?);
                    let mut archive = ZipArchive::new(file_reader)?;
                    let mut disc_file = archive.by_index(0)?;
                    let mut tmp_writer = BufWriter::new(&mut tmp);

                    msg_sender.send(Message::UpdateStatus(format!(
                        "🎮 Extracting {}",
                        disc_info.header.game_title_str()
                    )))?;
                    io::copy(&mut disc_file, &mut tmp_writer)?;
                }

                DiscReader::new_from_non_cloneable_read(tmp, &disc_opts)?
            } else {
                DiscReader::new(&disc_info.main_disc_path, &disc_opts)?
            };

            let mut overflow_writer = OverflowWriter::new(&file_path1, file_path2, must_split)?;

            let out_opts = get_output_format_opts(
                wii_output_format,
                gc_output_format,
                disc_info.header.is_wii(),
            );
            let writer = DiscWriter::new(disc, &out_opts)?;

            let process_opts = get_process_opts(
                scrub_update_partition
                    && disc_info.header.is_wii()
                    && wii_output_format == WiiOutputFormat::Wbfs,
            );

            let finalization = writer.process(
                |data, progress, total| {
                    overflow_writer.write_all(&data)?;

                    let _ = msg_sender.send(Message::UpdateStatus(format!(
                        "🎮 Converting {}  {:02.0}%",
                        disc_info.header.game_title_str(),
                        progress as f32 / total as f32 * 100.0,
                    )));

                    Ok(())
                },
                &process_opts,
            )?;

            if !finalization.header.is_empty() {
                overflow_writer.write_header(&finalization.header)?;
            }
        }

        if remove_sources_games {
            fs::remove_file(&disc_info.main_disc_path)?;
            if let Some(overflow_file) = &overflow_file {
                fs::remove_file(overflow_file)?;
            }
        }

        msg_sender.send(Message::NotifyInfo(format!(
            "🎮 {} Converted",
            disc_info.header.game_title_str()
        )))?;

        Ok(())
    });
}
