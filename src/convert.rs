// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    QueuedConversion, State,
    extensions::{ext_to_format, format_to_opts},
    id_map,
    util::{self, get_threads_num},
};
use anyhow::{Result, anyhow, bail};
use bitflags::bitflags;
use crc32fast::Hasher;
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, ProcessOptions, ScrubLevel},
};
use slint::{ToSharedString, Weak};
use split_write::SplitWriter;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Write},
    num::NonZeroUsize,
    path::PathBuf,
};
use zip::ZipArchive;

pub const SPLIT_SIZE: NonZeroUsize = NonZeroUsize::new(4_294_934_528).unwrap(); // 4 GiB - 32 KiB
const HEADER_SIZE: usize = 131_072;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ConversionFlags: i32 {
        const IS_FOR_DRIVE = 1;
        const IS_FAT32 = 1 << 1;
        const REMOVE_SOURCES = 1 << 2;
        const SCRUB_UPDATE = 1 << 3;
        const ALWAYS_SPLIT = 1 << 4;
        const IS_SCRUB_OPERATION = 1 << 5;
        const OUTPUT_WII_ISO = 1 << 6;
        const OUTPUT_GC_ISO = 1 << 7;
        const IS_WII = 1 << 8;
    }
}

pub struct Conversion {
    in_path: PathBuf,
    out_path: PathBuf,
    flags: ConversionFlags,
    game_id: String,
    game_title: String,
    disc_number: i32,
    files_to_remove: Vec<PathBuf>,
    weak: Weak<State<'static>>,
}

impl Conversion {
    pub fn new(q: &QueuedConversion, weak: Weak<State<'static>>) -> Self {
        let in_path = PathBuf::from(&q.in_path);
        let out_path = PathBuf::from(&q.out_path);
        let flags = ConversionFlags::from_bits(q.flags).unwrap();

        let game_id = q.game_id.to_string();
        let game_title = q.game_title.to_string();
        let disc_number = q.disc_number;

        let mut files_to_remove = Vec::new();
        if flags.contains(ConversionFlags::IS_FOR_DRIVE)
            && flags.contains(ConversionFlags::REMOVE_SOURCES)
        {
            files_to_remove.push(in_path.clone());
        }

        Self {
            in_path,
            out_path,
            flags,
            game_id,
            game_title,
            disc_number,
            files_to_remove,
            weak,
        }
    }

    pub fn perform(&mut self) -> Result<()> {
        self.unzip()?;

        let (processor_threads, preloader_threads) = get_threads_num();

        let disc_opts = DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads,
        };

        let must_split = self.flags.contains(ConversionFlags::IS_WII)
            && self.flags.contains(ConversionFlags::IS_FOR_DRIVE)
            && (self.flags.contains(ConversionFlags::ALWAYS_SPLIT)
                || self.flags.contains(ConversionFlags::IS_FAT32));

        // if we're converting a game for the wii, create the parent dir
        // we know we're converting for the wii as in-path is the mount point (a directory)
        let (parent, get_file_name, out_format) =
            if self.flags.contains(ConversionFlags::IS_FOR_DRIVE) {
                let display_title: &str =
                    id_map::get(&self.game_id).map_or(&self.game_title, |e| e.title);
                let sanitized_title = util::sanitize(display_title);

                let scrub_suffix = if self.flags.contains(ConversionFlags::IS_SCRUB_OPERATION) {
                    " SCRUB"
                } else {
                    ""
                };

                let parent = self
                    .out_path
                    .join(if self.flags.contains(ConversionFlags::IS_WII) {
                        "wbfs"
                    } else {
                        "games"
                    })
                    .join(format!(
                        "{} [{}]{}",
                        sanitized_title, &self.game_id, scrub_suffix
                    ));

                fs::create_dir_all(&parent)?;

                if self.flags.contains(ConversionFlags::IS_WII) {
                    let (f, out_format): (Box<dyn Fn(usize) -> String>, _) =
                        if self.flags.contains(ConversionFlags::OUTPUT_WII_ISO) {
                            let f: Box<dyn Fn(usize) -> String> = if must_split {
                                Box::new(|i| format!("{}.part{}.iso", &self.game_id, i))
                            } else {
                                Box::new(|_| format!("{}.iso", &self.game_id))
                            };

                            (f, Format::Iso)
                        } else {
                            let f: Box<dyn Fn(usize) -> String> = Box::new(|i| match i {
                                0 => format!("{}.wbfs", &self.game_id),
                                n => format!("{}.wbf{n}", &self.game_id),
                            });

                            (f, Format::Wbfs)
                        };

                    (parent, f, out_format)
                } else {
                    let (f, out_format): (Box<dyn Fn(usize) -> String>, _) =
                        if self.flags.contains(ConversionFlags::OUTPUT_GC_ISO) {
                            let f: Box<dyn Fn(usize) -> String> = match self.disc_number {
                                0 => Box::new(|_| "game.iso".to_string()),
                                n => Box::new(move |_| format!("disc{}.iso", n + 1)),
                            };

                            (f, Format::Iso)
                        } else {
                            let f: Box<dyn Fn(usize) -> String> = match self.disc_number {
                                0 => Box::new(|_| "game.ciso".to_string()),
                                n => Box::new(move |_| format!("disc{}.ciso", n + 1)),
                            };

                            (f, Format::Ciso)
                        };

                    (parent, f, out_format)
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

                let Some(out_format) = self.out_path.extension().and_then(ext_to_format) else {
                    bail!("Invalid output format");
                };

                let f: Box<dyn Fn(usize) -> String> = Box::new(move |_| filename.clone());

                (parent.to_path_buf(), f, out_format)
            };

        let scrub = if self.flags.contains(ConversionFlags::SCRUB_UPDATE) {
            ScrubLevel::UpdatePartition
        } else {
            ScrubLevel::None
        };

        let out_opts = format_to_opts(out_format);
        let process_opts = ProcessOptions {
            processor_threads,
            scrub,
            ..Default::default()
        };

        let split_size = if must_split { Some(SPLIT_SIZE) } else { None };

        let hash_path = parent.join(format!("{}.crc32", &self.game_id));

        let mut out_writer = BufWriter::with_capacity(
            32_768,
            SplitWriter::create(&parent, get_file_name, split_size)?,
        );

        let disc_reader = DiscReader::new(&self.in_path, &disc_opts)?;
        let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;
        let mut head_buffer = if self.flags.contains(ConversionFlags::IS_FOR_DRIVE) {
            Vec::with_capacity(HEADER_SIZE)
        } else {
            Vec::new()
        };
        let mut hasher = Hasher::new();

        let mut next_threshold = 0;
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                if self.flags.contains(ConversionFlags::IS_FOR_DRIVE) {
                    let remaining_in_head = HEADER_SIZE.saturating_sub(head_buffer.len());
                    if remaining_in_head > 0 {
                        let to_write = remaining_in_head.min(data.len());
                        head_buffer.extend_from_slice(&data[..to_write]);
                        hasher.update(&data[to_write..]);
                    } else {
                        hasher.update(&data);
                    }
                }

                if progress >= next_threshold {
                    let current_percentage = progress * 100 / total;
                    next_threshold = (current_percentage + 1) * total / 100;

                    let status = format!(
                        "⤒  Converting {}  {:02}%",
                        &self.game_title, current_percentage
                    );
                    let _ = self.weak.upgrade_in_event_loop(move |state| {
                        state.set_status(status.to_shared_string());
                    });
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

            if self.flags.contains(ConversionFlags::IS_FOR_DRIVE) {
                head_buffer[..finalization.header.len()].copy_from_slice(&finalization.header);
            }
        }

        split_writer.flush()?;

        if self.flags.contains(ConversionFlags::IS_FOR_DRIVE) {
            let mut final_hasher = Hasher::new();
            final_hasher.update(&head_buffer);
            final_hasher.combine(&hasher);
            let checksum = final_hasher.finalize();
            fs::write(hash_path, format!("{checksum:08x}"))?;
        }

        for path in &self.files_to_remove {
            let _ = fs::remove_file(path);
        }

        // If we're in a scrub operation, remove the original directory and rename the new one
        if self.flags.contains(ConversionFlags::IS_SCRUB_OPERATION)
            && let Some(og_parent) = self.in_path.parent()
        {
            fs::remove_dir_all(og_parent)?;

            let new_dirname = parent
                .file_name()
                .and_then(OsStr::to_str)
                .ok_or(anyhow!("Invalid filename"))?
                .trim_end_matches(" SCRUB");

            let new_dir_path = parent.with_file_name(new_dirname);

            fs::rename(parent, new_dir_path)?;
        }

        Ok(())
    }

    fn unzip(&mut self) -> Result<()> {
        let is_zip = self
            .in_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

        if !is_zip {
            return Ok(());
        }

        let status = format!("Unzipping {}", self.in_path.display());
        let _ = self.weak.upgrade_in_event_loop(move |state| {
            state.set_status(status.to_shared_string());
        });

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
            self.files_to_remove.push(new_in_path.clone());
        }

        self.in_path = new_in_path;

        Ok(())
    }
}
