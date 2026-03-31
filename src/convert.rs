// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    AppWindow, QueuedConversion, State,
    extensions::{ext_to_format, format_to_opts},
    id_map::ID_MAP,
    util::{self, get_threads_num},
};
use anyhow::{Result, anyhow, bail};
use bitflags::bitflags;
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, ProcessOptions, ScrubLevel},
};
use slint::{ComponentHandle, SharedString, ToSharedString, Weak};
use split_write::SplitWriter;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    num::NonZeroUsize,
    path::PathBuf,
    thread,
};
use zip::ZipArchive;

pub const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Flags: u8 {
        const IS_FAT32 = 1;
        const ALWAYS_SPLIT = 1 << 1;
        const REMOVE_SOURCES = 1 << 2;
        const SCRUB_UPDATE = 1 << 3;
    }
}

#[derive(Debug, Clone)]
struct Conversion {
    in_path: PathBuf,
    out_path: PathBuf,
    wii_output_format: Format,
    gc_output_format: Format,
    flags: Flags,
}

impl From<&QueuedConversion> for Conversion {
    fn from(value: &QueuedConversion) -> Self {
        let in_path = PathBuf::from(&value.in_path);
        let out_path = PathBuf::from(&value.out_path);

        let wii_output_format =
            ext_to_format(&value.conf.wii_output_format).unwrap_or(Format::Wbfs);
        let gc_output_format = ext_to_format(&value.conf.gc_output_format).unwrap_or(Format::Iso);

        let mut flags = Flags::empty();
        flags.set(Flags::IS_FAT32, value.is_fat32);
        flags.set(Flags::ALWAYS_SPLIT, value.conf.always_split);
        flags.set(Flags::REMOVE_SOURCES, value.conf.remove_sources_games);
        flags.set(Flags::SCRUB_UPDATE, value.conf.scrub_update_partition);

        Self {
            in_path,
            out_path,
            wii_output_format,
            gc_output_format,
            flags,
        }
    }
}

impl Conversion {
    #[allow(clippy::too_many_lines)]
    pub fn perform(mut self, weak: &Weak<AppWindow>) -> Result<()> {
        let is_for_drive = self.out_path.is_dir();

        let mut files_to_remove = Vec::new();
        if is_for_drive && self.flags.contains(Flags::REMOVE_SOURCES) {
            files_to_remove.push(self.in_path.clone());
        }

        if self
            .in_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            let status = format!("Unzipping {}", self.in_path.display()).to_shared_string();
            let _ =
                weak.upgrade_in_event_loop(move |app| app.global::<State<'_>>().set_status(status));

            let file = File::open(&self.in_path)?;
            let reader = BufReader::new(file);
            let mut archive = ZipArchive::new(reader)?;
            let mut archived_disc = archive.by_index(0)?;

            let Some(parent) = self.in_path.parent() else {
                bail!("No parent dir found");
            };

            let new_in_path = parent.join(archived_disc.name());
            if !new_in_path.exists() {
                let mut out = File::create(&new_in_path)?;
                io::copy(&mut archived_disc, &mut out)?;
                out.flush()?;
                files_to_remove.push(new_in_path.clone());
            }

            self.in_path = new_in_path;
        }

        let (processor_threads, preloader_threads) = get_threads_num();

        let disc_opts = DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads,
        };

        let disc_reader = DiscReader::new(&self.in_path, &disc_opts)?;
        let header = disc_reader.header();
        let is_wii = header.is_wii();
        let title = header.game_title_str().to_string();
        let id = header.game_id;
        let id_str = header.game_id_str().to_string();
        let disc_num = header.disc_num;

        let must_split =
            is_for_drive && is_wii && self.flags.intersects(Flags::ALWAYS_SPLIT | Flags::IS_FAT32);

        // if we're converting a game for the wii, create the parent dir
        // we know we're converting for the wii as in-path is the mount point (a directory)
        let (parent, get_file_name, out_format) = if is_for_drive {
            let display_title = ID_MAP.get(id).map_or(&title, |e| &e.title);
            let sanitized_title = util::sanitize(display_title);

            let parent = self
                .out_path
                .join(if is_wii { "wbfs" } else { "games" })
                .join(format!("{sanitized_title} [{id_str}]"));

            fs::create_dir_all(&parent)?;

            if is_wii {
                let f: Box<dyn Fn(usize) -> String> = match self.wii_output_format {
                    Format::Wbfs => Box::new(|i| match i {
                        0 => format!("{id_str}.wbfs"),
                        n => format!("{id_str}.wbf{n}"),
                    }),
                    _ => {
                        if must_split {
                            Box::new(|i| format!("{id_str}.part{i}.iso"))
                        } else {
                            Box::new(|_| format!("{id_str}.iso"))
                        }
                    }
                };

                (parent, f, self.wii_output_format)
            } else {
                let f: Box<dyn Fn(usize) -> String> = match disc_num {
                    0 => Box::new(|_| format!("game.{}", self.gc_output_format)),
                    n => Box::new(move |_| format!("disc{}.{}", n + 1, self.gc_output_format)),
                };

                (parent, f, self.gc_output_format)
            }
        } else {
            let Some(parent) = self.out_path.parent() else {
                bail!("No parent dir found");
            };

            let Some(filename) = self
                .out_path
                .file_name()
                .and_then(OsStr::to_str)
                .map(str::to_string)
            else {
                bail!("No filename found");
            };

            let Some(out_format) = self
                .out_path
                .extension()
                .and_then(OsStr::to_str)
                .and_then(ext_to_format)
            else {
                bail!("Invalid output format");
            };

            let f: Box<dyn Fn(usize) -> String> = Box::new(move |_| filename.clone());

            (parent.to_path_buf(), f, out_format)
        };

        let out_opts = format_to_opts(out_format);
        let process_opts = ProcessOptions {
            processor_threads,
            digest_crc32: true,
            digest_md5: false,
            digest_sha1: true,
            digest_xxh64: true,
            scrub: if self.flags.contains(Flags::SCRUB_UPDATE) {
                ScrubLevel::UpdatePartition
            } else {
                ScrubLevel::None
            },
        };

        let split_size = if must_split { Some(SPLIT_SIZE) } else { None };

        let mut out_writer = BufWriter::with_capacity(
            32_768,
            SplitWriter::create(parent, get_file_name, split_size)?,
        );

        let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;

        let mut prev_percentage = 100;
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                let progress_percentage = progress * 100 / total;
                if progress_percentage != prev_percentage {
                    let status = format!("⤒ Converting {title}  {progress_percentage:02}%")
                        .to_shared_string();
                    let _ = weak.upgrade_in_event_loop(move |app| {
                        app.global::<State<'_>>().set_status(status);
                    });

                    prev_percentage = progress_percentage;
                }

                Ok(())
            },
            &process_opts,
        )?;

        let mut split_writer = out_writer
            .into_inner()
            .map_err(|_| anyhow!("Failed to get inner split writer"))?;

        if !finalization.header.is_empty() {
            split_writer.write_header(&finalization.header)?;
        }

        split_writer.flush()?;

        for path in files_to_remove {
            let _ = fs::remove_file(path);
        }

        Ok(())
    }
}

impl QueuedConversion {
    pub fn run(&self, weak: Weak<AppWindow>) {
        weak.upgrade()
            .unwrap()
            .global::<State<'_>>()
            .set_is_converting(true);

        let conv = Conversion::from(self);

        let _ = thread::spawn(move || {
            let _ = conv.perform(&weak);

            let _ = weak.upgrade_in_event_loop(|app| {
                app.global::<State<'_>>().set_is_converting(false);
                app.global::<State<'_>>().set_status(SharedString::new());
            });
        });
    }
}
