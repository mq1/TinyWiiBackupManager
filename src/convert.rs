// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::disc_info::is_worth_stripping;
use crate::extensions::{ext_to_format, format_to_opts};
use crate::messages::Message;
use crate::overflow_reader::get_main_file;
use crate::{
    disc_info::DiscInfo,
    overflow_reader::{OverflowReader, get_overflow_file},
    overflow_writer::OverflowWriter,
    util::{self, can_write_over_4gb},
};
use anyhow::{anyhow, bail};
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use sanitize_filename::sanitize;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::PathBuf;
use std::{fs, io::Write};
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

pub fn spawn_strip_game_task(app: &App, disc_info: DiscInfo) {
    let disc_opts = get_disc_opts();
    let always_split = app.config.contents.always_split;

    app.task_processor.spawn(move |msg_sender| {
        let overflow_file = get_overflow_file(&disc_info.main_disc_path);

        let old_name = disc_info
            .game_dir
            .file_name()
            .ok_or(anyhow!("No file name"))?
            .to_str()
            .ok_or(anyhow!("Invalid file name"))?;

        let new_name = format!("{}.new", old_name);

        let parent_dir = disc_info
            .game_dir
            .parent()
            .ok_or(anyhow!("No parent directory"))?;

        let dir_path = parent_dir.join(new_name);
        let out_path = dir_path.join(format!("{}.wbfs", disc_info.id.as_str()));

        let disc = if let Some(overflow_file) = &overflow_file {
            let reader = OverflowReader::new(&disc_info.main_disc_path, overflow_file)?;
            DiscReader::new_from_non_cloneable_read(reader, &disc_opts)?
        } else {
            DiscReader::new(&disc_info.main_disc_path, &disc_opts)?
        };

        let mut overflow_writer = OverflowWriter::new(&out_path, always_split)?;

        let out_opts = FormatOptions::new(Format::Wbfs);
        let writer = DiscWriter::new(disc, &out_opts)?;

        let process_opts = get_process_opts(true);

        let finalization = writer.process(
            |data, progress, total| {
                overflow_writer.write_all(&data)?;

                let _ = msg_sender.send(Message::UpdateStatus(format!(
                    "{} Converting {}  {:02}%",
                    egui_phosphor::regular::FLOW_ARROW,
                    &disc_info.title,
                    progress * 100 / total,
                )));

                Ok(())
            },
            &process_opts,
        )?;

        if !finalization.header.is_empty() {
            overflow_writer.write_header(&finalization.header)?;
        }

        // Remove the original game directory
        fs::remove_dir_all(&disc_info.game_dir)?;

        // Rename the new directory to the original game directory name
        fs::rename(dir_path, disc_info.game_dir)?;

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
            && let Ok(disc) = DiscReader::new(&path, &get_disc_opts())
            && is_worth_stripping(&disc)
            && let Ok(disc_info) = DiscInfo::from_path(path)
        {
            spawn_strip_game_task(app, disc_info);
            at_least_one_stripped = true;
        }
    }

    at_least_one_stripped
}

pub fn spawn_conv_game_task(app: &App, in_path: PathBuf, out_path: PathBuf) {
    let scrub_update_partition = app.config.contents.scrub_update_partition;
    let always_split = app.config.contents.always_split;

    app.task_processor.spawn(move |msg_sender| {
        if out_path.exists() {
            bail!("{} already exists", out_path.display());
        }

        let mut tmp = None;

        // If the disc is a ZIP file, extract it to a temporary file
        if in_path.extension() == Some(OsStr::new("zip")) {
            let mut tmp = tmp.insert(tempfile()?);
            let zip_file_reader = BufReader::new(File::open(&in_path)?);
            let mut archive = ZipArchive::new(zip_file_reader)?;
            let mut disc_reader = archive.by_index(0)?;
            let mut tmp_writer = BufWriter::new(&mut tmp);

            let mut buf = [0; 8192];
            let mut progress = 0u64;
            let total = disc_reader.size();
            loop {
                let n = disc_reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                tmp_writer.write_all(&buf[..n])?;
                progress += n as u64;

                msg_sender.send(Message::UpdateStatus(format!(
                    "{} Extracting {}  {:02}%",
                    egui_phosphor::regular::FILE_ZIP,
                    in_path.display(),
                    progress * 100 / total
                )))?;
            }
        }

        let disc_opts = get_disc_opts();

        let out_format = out_path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(ext_to_format)
            .ok_or(anyhow!("Invalid output file extension"))?;

        let overflow_file = get_overflow_file(&in_path);

        let disc = if let Some(tmp) = tmp {
            DiscReader::new_from_non_cloneable_read(tmp, &disc_opts)?
        } else if let Some(overflow_file) = &overflow_file {
            let reader = OverflowReader::new(&in_path, overflow_file)?;
            DiscReader::new_from_non_cloneable_read(reader, &disc_opts)?
        } else {
            DiscReader::new(&in_path, &disc_opts)?
        };
        let game_title = disc.header().game_title_str().to_string();

        let mut overflow_writer = OverflowWriter::new(&out_path, always_split)?;
        let out_opts = format_to_opts(out_format);
        let disc_writer = DiscWriter::new(disc, &out_opts)?;
        let process_opts = get_process_opts(scrub_update_partition);

        let finalization = disc_writer.process(
            |data, progress, total| {
                overflow_writer.write_all(&data)?;

                let _ = msg_sender.send(Message::UpdateStatus(format!(
                    "{} Converting {}  {:02}%",
                    egui_phosphor::regular::FLOW_ARROW,
                    &game_title,
                    progress * 100 / total
                )));

                Ok(())
            },
            &process_opts,
        )?;

        if !finalization.header.is_empty() {
            overflow_writer.write_header(&finalization.header)?;
        }

        Ok(())
    });
}

pub fn spawn_add_game_task(app: &App, in_path: PathBuf, should_download_covers: bool) {
    let mount_point = app.config.contents.mount_point.clone();
    let scrub_update_partition = app.config.contents.scrub_update_partition;
    let always_split = app.config.contents.always_split;
    let wii_output_format = app.config.contents.wii_output_format;
    let gc_output_format = app.config.contents.gc_output_format;
    let remove_sources = app.config.contents.remove_sources_games;

    app.task_processor.spawn(move |msg_sender| {
        let mut tmp = None;

        // If the disc is a ZIP file, extract it to a temporary file
        if in_path.extension() == Some(OsStr::new("zip")) {
            let mut tmp = tmp.insert(tempfile()?);
            let zip_file_reader = BufReader::new(File::open(&in_path)?);
            let mut archive = ZipArchive::new(zip_file_reader)?;
            let mut disc_reader = archive.by_index(0)?;
            let mut tmp_writer = BufWriter::new(&mut tmp);

            let mut buf = [0; 8192];
            let mut progress = 0u64;
            let total = disc_reader.size();
            loop {
                let n = disc_reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                tmp_writer.write_all(&buf[..n])?;
                progress += n as u64;

                msg_sender.send(Message::UpdateStatus(format!(
                    "{} Extracting {}  {:02}%",
                    egui_phosphor::regular::FILE_ZIP,
                    in_path.display(),
                    progress * 100 / total
                )))?;
            }
        }

        let disc_opts = get_disc_opts();
        let overflow_file = get_overflow_file(&in_path);

        let disc = if let Some(tmp) = tmp {
            DiscReader::new_from_non_cloneable_read(tmp, &disc_opts)?
        } else if let Some(overflow_file) = &overflow_file {
            let reader = OverflowReader::new(&in_path, overflow_file)?;
            DiscReader::new_from_non_cloneable_read(reader, &disc_opts)?
        } else {
            DiscReader::new(&in_path, &disc_opts)?
        };

        let disc_header = disc.header();
        let game_id = disc_header.game_id_str().to_string();
        let game_title = disc_header.game_title_str().to_string();
        let is_wii = disc_header.is_wii();

        let out_dir = mount_point
            .join(if is_wii { "wbfs" } else { "games" })
            .join(format!("{} [{}]", sanitize(&game_title), &game_id));

        let out_file_name = if is_wii {
            if wii_output_format == Format::Wbfs {
                format!("{}.wbfs", &game_id)
            } else if always_split || can_write_over_4gb(&mount_point).is_err() {
                format!("{}.part0.iso", &game_id)
            } else {
                format!("{}.iso", &game_id)
            }
        } else if wii_output_format == Format::Ciso {
            if disc_header.disc_num == 0 {
                "game.ciso".to_string()
            } else {
                format!("disc{}.ciso", disc_header.disc_num + 1)
            }
        } else if disc_header.disc_num == 0 {
            "game.iso".to_string()
        } else {
            format!("disc{}.iso", disc_header.disc_num + 1)
        };

        let out_path = out_dir.join(out_file_name);
        if out_path.exists() {
            return Ok(());
        }

        let mut overflow_writer = OverflowWriter::new(&out_path, always_split)?;

        let out_format = if is_wii {
            wii_output_format
        } else {
            gc_output_format
        };

        let out_opts = format_to_opts(out_format);
        let disc_writer = DiscWriter::new(disc, &out_opts)?;
        let process_opts = get_process_opts(scrub_update_partition);

        let finalization = disc_writer.process(
            |data, progress, total| {
                overflow_writer.write_all(&data)?;

                let _ = msg_sender.send(Message::UpdateStatus(format!(
                    "{} Converting {}  {:02}%",
                    egui_phosphor::regular::FLOW_ARROW,
                    &game_title,
                    progress * 100 / total
                )));

                Ok(())
            },
            &process_opts,
        )?;

        if !finalization.header.is_empty() {
            overflow_writer.write_header(&finalization.header)?;
        }

        if remove_sources {
            fs::remove_file(&in_path)?;
            if let Some(overflow_file) = &overflow_file {
                fs::remove_file(overflow_file)?;
            }
        }

        msg_sender.send(Message::TriggerRefreshGames(should_download_covers))?;

        Ok(())
    });
}

pub fn spawn_add_games_task(app: &App, discs: Box<[DiscInfo]>) {
    let last_i = discs.len() - 1;

    for (i, disc_info) in discs.into_iter().enumerate() {
        spawn_add_game_task(app, disc_info.main_disc_path, i == last_i);
    }
}
