// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, GcOutputFormat, WiiOutputFormat},
    errors::Error,
    games::disc_reader::get_disc_reader,
    util::{drive_info::DriveInfo, misc::OPTIMAL_THREADS},
};
use iced::task::{Straw, sipper};
use nod::{
    common::Format,
    write::{DiscWriter, FormatOptions, ProcessOptions, ScrubLevel},
};
use split_write::SplitWriter;
use std::{
    borrow::Cow,
    ffi::OsStr,
    fs,
    io::{BufWriter, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};
use tap::Pipe;
use which_fs::FsKind;

const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB

fn perform_blocking(
    disc_path: PathBuf,
    config: Config,
    is_fat32: bool,
    tx: smol::channel::Sender<String>,
) -> impl FnOnce() -> Result<(), Error> {
    move || {
        let disc_reader = get_disc_reader(&disc_path)?;

        let disc_header = disc_reader.header();
        let is_wii = disc_header.is_wii();
        let game_id = disc_header.game_id_str().to_string();
        let game_title = disc_header.game_title_str().to_string();
        let disc_num = usize::from(disc_header.disc_num);

        let must_split = is_wii && (is_fat32 || config.always_split);
        let split_size = if must_split { Some(SPLIT_SIZE) } else { None };

        let parent_dir_name = if is_wii { "wbfs" } else { "games" };
        let parent_dir = config.mount_point.join(parent_dir_name);
        let game_dir = make_game_dir(&parent_dir, &game_id, &game_title)?;

        let out_writer = SplitWriter::create(
            &game_dir,
            |part| get_filename(&game_id, is_wii, part, disc_num, &config, must_split),
            split_size,
        )?;
        let mut out_writer = BufWriter::with_capacity(0x8000, out_writer);

        let out_format = get_out_format(is_wii, &config);
        let disc_writer = DiscWriter::new(disc_reader, &FormatOptions::new(out_format))?;

        let mut prev_percentage = 100;
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                let progress_percentage = progress * 100 / total;
                if progress_percentage != prev_percentage {
                    let status = format!("⤓  Importing {game_title}  {progress_percentage:02}%");
                    let _ = tx.try_send(status);

                    prev_percentage = progress_percentage;
                }

                Ok(())
            },
            &ProcessOptions {
                processor_threads: OPTIMAL_THREADS.processor,
                scrub: ScrubLevel::None,
                digest_crc32: true,
                digest_md5: false,
                digest_sha1: true,
                digest_xxh64: true,
            },
        )?;

        let mut out_writer = out_writer.into_inner()?;

        if !finalization.header.is_empty() {
            out_writer.write_header(&finalization.header)?;
        }

        out_writer.flush()?;
        Ok(())
    }
}

pub fn import_game(
    path: PathBuf,
    config: Config,
    drive_info: Option<DriveInfo>,
) -> impl Straw<(), String, Error> {
    let is_fat32 = drive_info.is_some_and(|drive_info| drive_info.fs_kind == FsKind::Fat32);

    sipper(async move |mut sender| {
        let filename = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or(Error::InvalidFilename)?;

        sender.send(format!("›  Opening {filename}")).await;

        let remove_sources = config.remove_sources_games;

        let (tx, rx) = smol::channel::bounded(1);

        let handle = perform_blocking(path.clone(), config, is_fat32, tx).pipe(std::thread::spawn);

        while let Ok(msg) = rx.recv().await {
            sender.send(msg).await;
        }

        handle.join().expect("Failed to join thread")?;

        if remove_sources {
            smol::fs::remove_file(&path).await?;
        }

        Ok(())
    })
}

fn get_filename(
    game_id: &str,
    is_wii: bool,
    part: usize,
    disc_num: usize,
    config: &Config,
    must_split: bool,
) -> String {
    if is_wii {
        match config.wii_output_format {
            WiiOutputFormat::Iso => {
                if must_split {
                    format!("{game_id}.part{part}.iso")
                } else {
                    format!("{game_id}.iso")
                }
            }
            WiiOutputFormat::Wbfs => match part {
                0 => format!("{game_id}.wbfs"),
                n => format!("{game_id}.wbf{n}"),
            },
        }
    } else {
        match config.gc_output_format {
            GcOutputFormat::Iso => match disc_num {
                0 => "game.iso".to_string(),
                n => format!("disc{}.iso", n + 1),
            },

            GcOutputFormat::Ciso => match disc_num {
                0 => "game.ciso".to_string(),
                n => format!("disc{}.ciso", n + 1),
            },
        }
    }
}

fn get_out_format(is_wii: bool, config: &Config) -> Format {
    match (is_wii, config.wii_output_format, config.gc_output_format) {
        (true, WiiOutputFormat::Iso, _) | (false, _, GcOutputFormat::Iso) => Format::Iso,
        (true, WiiOutputFormat::Wbfs, _) => Format::Wbfs,
        (false, _, GcOutputFormat::Ciso) => Format::Ciso,
    }
}

fn is_valid_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || " !#$%&'()+,-.;=@^_`{}~".contains(c)
}

pub fn sanitize_title(ascii_title: &str) -> Cow<'static, str> {
    ascii_title
        .chars()
        .skip_while(|c| !c.is_ascii_alphanumeric())
        .filter(|&c| is_valid_char(c))
        .take(64)
        .collect::<String>()
        .trim_end()
        .pipe(|s| match s.is_empty() {
            true => Cow::Borrowed("game"),
            false => Cow::Owned(s.into()),
        })
}

pub fn make_game_dir_name(game_id: impl AsRef<str>, fallback_title: &str) -> String {
    let game_id = game_id.as_ref();

    twbm_idmap::get_title(game_id)
        .unwrap_or(fallback_title)
        .pipe(sanitize_title)
        .pipe(|title| format!("{title} [{game_id}]"))
}

fn make_game_dir(
    base_dir: &Path,
    game_id: &str,
    fallback_title: &str,
) -> Result<PathBuf, std::io::Error> {
    make_game_dir_name(game_id, fallback_title)
        .pipe(|name| base_dir.join(name))
        .pipe(|path| fs::create_dir_all(&path).map(|_| path))
}
