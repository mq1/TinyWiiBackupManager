// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::disc_info::is_worth_stripping;
use crate::messages::Message;
use crate::overflow_reader::get_main_file;
use crate::{
    config::{GcOutputFormat, WiiOutputFormat},
    disc_info::DiscInfo,
    overflow_reader::{OverflowReader, get_overflow_file},
    overflow_writer::OverflowWriter,
    util::{self, can_write_over_4gb},
};
use anyhow::anyhow;
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use sanitize_filename::sanitize;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::{
    fs::{self},
    io,
    io::Write,
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
    let last_i = discs.len() - 1;

    for (i, disc) in discs.into_iter().enumerate() {
        spawn_convert_game_task(
            app,
            disc.main_disc_path,
            app.config.contents.mount_point.clone(),
            true,
            i == last_i,
        );
    }
}

// subdir is "wbfs" or "games""
pub fn spawn_convert_game_task(
    app: &App,
    main_disc_path: PathBuf,
    mut dest_dir: PathBuf,
    mk_subdir: bool,
    download_covers: bool,
) {
    let disc_opts = get_disc_opts();

    let wii_output_format = app.config.contents.wii_output_format;
    let gc_output_format = app.config.contents.gc_output_format;
    let scrub_update_partition = app.config.contents.scrub_update_partition;
    let remove_sources_games = app.config.contents.remove_sources_games;

    let must_split = app.config.contents.always_split
        || can_write_over_4gb(&app.config.contents.mount_point).is_err();

    app.task_processor.spawn(move |msg_sender| {
        let mut tmp = None;

        // If the disc is a ZIP file, extract it to a temporary file
        if main_disc_path.extension() == Some(OsStr::new("zip")) {
            let mut tmp = tmp.insert(tempfile()?);
            let file_reader = BufReader::new(File::open(&main_disc_path)?);
            let mut archive = ZipArchive::new(file_reader)?;
            let mut disc_file = archive.by_index(0)?;
            let mut tmp_writer = BufWriter::new(&mut tmp);

            msg_sender.send(Message::UpdateStatus(format!(
                "{} Extracting {}",
                egui_phosphor::regular::FILE_ZIP,
                main_disc_path.display()
            )))?;

            io::copy(&mut disc_file, &mut tmp_writer)?;
        }

        let overflow_file = get_overflow_file(&main_disc_path);

        let disc = if let Some(tmp) = tmp {
            DiscReader::new_from_non_cloneable_read(tmp, &disc_opts)?
        } else if let Some(overflow_file) = &overflow_file {
            let reader = OverflowReader::new(&main_disc_path, overflow_file)?;
            DiscReader::new_from_non_cloneable_read(reader, &disc_opts)?
        } else {
            DiscReader::new(&main_disc_path, &disc_opts)?
        };
        let disc_header = disc.header();

        let is_wii = disc_header.is_wii();
        let disc_num = disc_header.disc_num;
        let game_id = disc_header.game_id_str().to_string();
        let game_title = disc_header.game_title_str().to_string();

        if mk_subdir {
            dest_dir = dest_dir.join(if is_wii { "wbfs" } else { "games" });
        }
        let dir_path = dest_dir.join(format!("{} [{}]", sanitize(&game_title), &game_id));
        fs::create_dir_all(&dir_path)?;

        let file_name1 = match (
            is_wii,
            wii_output_format,
            gc_output_format,
            must_split,
            disc_num,
        ) {
            (true, WiiOutputFormat::Wbfs, _, _, _) => &format!("{}.wbfs", &game_id),
            (true, WiiOutputFormat::Iso, _, true, _) => &format!("{}.part0.iso", &game_id),
            (true, WiiOutputFormat::Iso, _, false, _) => &format!("{}.iso", &game_id),
            (false, _, GcOutputFormat::Iso, _, 0) => "game.iso",
            (false, _, GcOutputFormat::Iso, _, n) => &format!("disc{}.iso", n + 1),
            (false, _, GcOutputFormat::Ciso, _, 0) => "game.ciso",
            (false, _, GcOutputFormat::Ciso, _, n) => &format!("disc{}.ciso", n + 1),
        };

        let file_path1 = dir_path.join(file_name1);

        if file_path1.exists() {
            return Ok(());
        }

        let file_name2 = match wii_output_format {
            WiiOutputFormat::Wbfs => &format!("{}.wbf1", &game_id),
            WiiOutputFormat::Iso => &format!("{}.part1.iso", &game_id),
        };
        let file_path2 = dir_path.join(file_name2);

        let mut overflow_writer = OverflowWriter::new(&file_path1, file_path2, must_split)?;

        let out_opts = get_output_format_opts(wii_output_format, gc_output_format, is_wii);
        let writer = DiscWriter::new(disc, &out_opts)?;

        let process_opts = get_process_opts(
            scrub_update_partition && is_wii && wii_output_format == WiiOutputFormat::Wbfs,
        );

        let finalization = writer.process(
            |data, progress, total| {
                overflow_writer.write_all(&data)?;

                let _ = msg_sender.send(Message::UpdateStatus(format!(
                    "{} Converting {}  {:02.0}%",
                    egui_phosphor::regular::FLOW_ARROW,
                    &game_title,
                    progress as f32 / total as f32 * 100.0,
                )));

                Ok(())
            },
            &process_opts,
        )?;

        if !finalization.header.is_empty() {
            overflow_writer.write_header(&finalization.header)?;
        }

        if remove_sources_games {
            fs::remove_file(&main_disc_path)?;
            if let Some(overflow_file) = &overflow_file {
                fs::remove_file(overflow_file)?;
            }
        }

        msg_sender.send(Message::NotifyInfo(format!(
            "{} {} Converted",
            egui_phosphor::regular::FLOW_ARROW,
            game_title
        )))?;
        msg_sender.send(Message::TriggerRefreshGames(download_covers))?;

        Ok(())
    });
}

pub fn spawn_strip_game_task(app: &App, game_dir: PathBuf) {
    let disc_opts = get_disc_opts();

    let must_split = app.config.contents.always_split
        || can_write_over_4gb(&app.config.contents.mount_point).is_err();

    app.task_processor.spawn(move |msg_sender| {
        let disc_info = DiscInfo::from_game_dir(&game_dir)?;

        let overflow_file = get_overflow_file(&disc_info.main_disc_path);

        let old_name = game_dir
            .file_name()
            .ok_or(anyhow!("No file name"))?
            .to_str()
            .ok_or(anyhow!("Invalid file name"))?;

        let new_name = format!("{}.new", old_name);

        let parent_dir = game_dir.parent().ok_or(anyhow!("No parent directory"))?;

        let dir_path = parent_dir.join(new_name);

        let file_name1 = format!("{}.wbfs", disc_info.id.as_str());
        let file_path1 = dir_path.join(file_name1);

        let file_name2 = format!("{}.wbf1", disc_info.id.as_str());
        let file_path2 = dir_path.join(file_name2);

        fs::create_dir_all(&dir_path)?;
        {
            let disc = if let Some(overflow_file) = &overflow_file {
                let reader = OverflowReader::new(&disc_info.main_disc_path, overflow_file)?;
                DiscReader::new_from_non_cloneable_read(reader, &disc_opts)?
            } else {
                DiscReader::new(&disc_info.main_disc_path, &disc_opts)?
            };

            let mut overflow_writer = OverflowWriter::new(&file_path1, file_path2, must_split)?;

            let out_opts = FormatOptions::new(Format::Wbfs);
            let writer = DiscWriter::new(disc, &out_opts)?;

            let process_opts = get_process_opts(true);

            let finalization = writer.process(
                |data, progress, total| {
                    overflow_writer.write_all(&data)?;

                    let _ = msg_sender.send(Message::UpdateStatus(format!(
                        "{} Converting {}  {:02.0}%",
                        egui_phosphor::regular::FLOW_ARROW,
                        &disc_info.title,
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

        // Remove the original game directory
        fs::remove_dir_all(&game_dir)?;

        // Rename the new directory to the original game directory name
        fs::rename(dir_path, game_dir)?;

        msg_sender.send(Message::NotifyInfo(format!(
            "{} {} Converted (without update partition)",
            egui_phosphor::regular::FLOW_ARROW,
            &disc_info.title
        )))?;
        msg_sender.send(Message::TriggerRefreshGames(false))?;

        Ok(())
    });
}

pub fn spawn_strip_all_games_tasks(app: &mut App) -> bool {
    let mut at_least_one_stripped = false;

    for game in &app.games {
        if let Some(path) = get_main_file(&game.path)
            && let Ok(file) = File::open(path)
            && is_worth_stripping(file)
        {
            spawn_strip_game_task(app, game.path.clone());
            at_least_one_stripped = true;
        }
    }

    at_least_one_stripped
}
