// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::App,
    config::{Config, GcOutputFormat, WiiOutputFormat},
    disc_info::DiscInfo,
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
    time::Instant,
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
        digest_md5: false,
        digest_sha1: false,
        digest_xxh64: false,
        scrub,
    }
}

fn get_output_format_opts(config: &Config, is_wii: bool) -> FormatOptions {
    if is_wii {
        if config.contents.wii_output_format == WiiOutputFormat::Wbfs {
            FormatOptions::new(Format::Wbfs)
        } else {
            FormatOptions::new(Format::Iso)
        }
    } else if config.contents.gc_output_format == GcOutputFormat::Iso {
        FormatOptions::new(Format::Iso)
    } else {
        FormatOptions::new(Format::Ciso)
    }
}

pub fn spawn_add_games_task(app: &App, discs: Box<[DiscInfo]>) {
    let config = app.config.clone();
    let disc_opts = get_disc_opts();
    let must_split = app.config.contents.always_split
        || can_write_over_4gb(&app.config.contents.mount_point).is_err();

    app.task_processor.spawn(move |msg_sender| {
        let len = discs.len();
        for (i, disc_info) in discs.into_iter().enumerate() {
            let dir_path = config
                .contents
                .mount_point
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
                config.contents.wii_output_format,
                config.contents.gc_output_format,
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
                (false, _, GcOutputFormat::Iso, _, n) => &format!("disc{n}.iso"),
                (false, _, GcOutputFormat::Ciso, _, 0) => "game.ciso",
                (false, _, GcOutputFormat::Ciso, _, n) => &format!("disc{n}.ciso"),
            };

            let overflow_file = get_overflow_file(&disc_info.main_disc_path);

            fs::create_dir_all(&dir_path)?;

            let start_instant = Instant::now();
            log::info!("Converting {}", disc_info.header.game_title_str());
            {
                let disc = if let Some(overflow_file) = &overflow_file {
                    let reader = OverflowReader::new(&disc_info.main_disc_path, overflow_file)?;
                    DiscReader::new_stream(Box::new(reader), &disc_opts)?
                } else {
                    DiscReader::new(&disc_info.main_disc_path, &disc_opts)?
                };

                let path1 = dir_path.join(file_name1);
                let mut out1 = BufWriter::new(File::create(&path1)?);

                let file_name2 = match config.contents.wii_output_format {
                    WiiOutputFormat::Wbfs => &format!("{}.wbf1", disc_info.header.game_id_str()),
                    WiiOutputFormat::Iso => {
                        &format!("{}.part1.iso", disc_info.header.game_id_str())
                    }
                };
                let path2 = dir_path.join(file_name2);
                let mut out2: Option<BufWriter<File>> = None;

                let out_opts = get_output_format_opts(&config, disc_info.header.is_wii());
                let writer = DiscWriter::new(disc, &out_opts)?;

                let process_opts = get_process_opts(
                    config.contents.scrub_update_partition
                        && disc_info.header.is_wii()
                        && config.contents.wii_output_format == WiiOutputFormat::Wbfs,
                );

                let finalization = writer.process(
                    |data, progress, total| {
                        // get position
                        let pos = out1.stream_position()?;

                        // write data to out1, or overflow to out2
                        if let Some(out2) = out2.as_mut() {
                            out2.write_all(&data)?;
                        } else if disc_info.header.is_wii()
                            && must_split
                            && pos + data.len() as u64 > SPLIT_SIZE
                        {
                            let mut writer = BufWriter::new(File::create(&path2)?);
                            writer.write_all(&data)?;
                            out2 = Some(writer);
                        } else {
                            out1.write_all(&data)?;
                        }

                        let _ = msg_sender.send(BackgroundMessage::UpdateStatus(format!(
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
                    out1.rewind()?;
                    out1.write_all(&finalization.header)?;
                }
            }
            log::info!(
                "Converted {} in {:.2}s",
                disc_info.header.game_title_str(),
                start_instant.elapsed().as_secs_f32()
            );

            if config.contents.remove_sources_games {
                fs::remove_file(&disc_info.main_disc_path)?;
                if let Some(overflow_file) = &overflow_file {
                    fs::remove_file(overflow_file)?;
                }
            }

            msg_sender.send(BackgroundMessage::NotifyInfo(format!(
                "🎮 Added {}",
                disc_info.header.game_title_str()
            )))?;
            msg_sender.send(BackgroundMessage::TriggerRefreshGames)?;
        }

        Ok(())
    });
}
