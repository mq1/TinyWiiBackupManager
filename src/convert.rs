// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    QueuedConversion, State,
    extensions::{ext_to_format, format_to_opts},
    id_map::ID_MAP,
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
const MAX_HEADER_SIZE: usize = 66_064; // WBFS header

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ConversionFlags: i32 {
        const IS_FOR_DRIVE = 1;
        const IS_FAT32 = 1 << 1;
        const REMOVE_SOURCES = 1 << 2;
        const SCRUB_UPDATE = 1 << 3;
        const ALWAYS_SPLIT = 1 << 4;
    }
}

pub struct Conversion {
    in_path: PathBuf,
    out_path: PathBuf,
    wii_output_format: Format,
    gc_output_format: Format,
    flags: ConversionFlags,
}

impl TryFrom<&QueuedConversion> for Conversion {
    type Error = anyhow::Error;

    fn try_from(q: &QueuedConversion) -> Result<Self, Self::Error> {
        let in_path = PathBuf::from(&q.in_path);
        let out_path = PathBuf::from(&q.out_path);
        let wii_output_format =
            ext_to_format(&q.wii_output_format).ok_or(anyhow!("Invalid output format"))?;
        let gc_output_format =
            ext_to_format(&q.gc_output_format).ok_or(anyhow!("Invalid output format"))?;
        let flags =
            ConversionFlags::from_bits(q.flags).ok_or(anyhow!("Invalid conversion flags"))?;

        Ok(Self {
            in_path,
            out_path,
            wii_output_format,
            gc_output_format,
            flags,
        })
    }
}

impl Conversion {
    #[allow(clippy::too_many_lines)]
    pub fn perform(&mut self, weak: &Weak<State<'static>>) -> Result<()> {
        let mut files_to_remove = Vec::new();
        if self
            .flags
            .contains(ConversionFlags::IS_FOR_DRIVE | ConversionFlags::REMOVE_SOURCES)
        {
            files_to_remove.push(self.in_path.clone());
        }

        if self
            .in_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
        {
            let status = format!("Unzipping {}", self.in_path.display()).to_shared_string();
            let _ = weak.upgrade_in_event_loop(move |state| state.set_status(status));

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

        let meta = {
            let file = File::open(&self.in_path)?;
            let mut reader = BufReader::new(file);
            wii_disc_info::Meta::read(&mut reader)?
        };

        let must_split = meta.is_wii()
            && self.flags.contains(ConversionFlags::IS_FOR_DRIVE)
            && self
                .flags
                .intersects(ConversionFlags::ALWAYS_SPLIT | ConversionFlags::IS_FAT32);

        // if we're converting a game for the wii, create the parent dir
        // we know we're converting for the wii as in-path is the mount point (a directory)
        let (parent, get_file_name, out_format) =
            if self.flags.contains(ConversionFlags::IS_FOR_DRIVE) {
                let display_title = ID_MAP
                    .get(meta.game_id())
                    .map_or(meta.game_title(), |e| &e.title);
                let sanitized_title = util::sanitize(display_title);

                let parent = self
                    .out_path
                    .join(if meta.is_wii() { "wbfs" } else { "games" })
                    .join(format!("{} [{}]", sanitized_title, meta.game_id()));

                fs::create_dir_all(&parent)?;

                if meta.is_wii() {
                    let f: Box<dyn Fn(usize) -> String> = match self.wii_output_format {
                        Format::Wbfs => Box::new(|i| match i {
                            0 => format!("{}.wbfs", meta.game_id()),
                            n => format!("{}.wbf{n}", meta.game_id()),
                        }),
                        Format::Iso => {
                            if must_split {
                                Box::new(|i| format!("{}.part{}.iso", meta.game_id(), i))
                            } else {
                                Box::new(|_| format!("{}.iso", meta.game_id()))
                            }
                        }
                        _ => {
                            bail!("Invalid output format");
                        }
                    };

                    (parent, f, self.wii_output_format)
                } else {
                    let f: Box<dyn Fn(usize) -> String> = match self.gc_output_format {
                        Format::Iso => match meta.disc_number() {
                            0 => Box::new(|_| "game.iso".to_string()),
                            n => Box::new(move |_| format!("disc{}.iso", n + 1)),
                        },
                        Format::Ciso => match meta.disc_number() {
                            0 => Box::new(|_| "game.ciso".to_string()),
                            n => Box::new(move |_| format!("disc{}.ciso", n + 1)),
                        },
                        _ => {
                            bail!("Invalid output format");
                        }
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

        let hash_path = parent.join(format!("{}.crc32", meta.game_id()));

        let mut out_writer = BufWriter::with_capacity(
            32_768,
            SplitWriter::create(parent, get_file_name, split_size)?,
        );

        let disc_reader = DiscReader::new(&self.in_path, &disc_opts)?;
        let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;
        let mut head_buffer = Vec::with_capacity(MAX_HEADER_SIZE);
        let mut hasher = Hasher::new();

        let mut prev_percentage = 100;
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                let data_len = data.len();
                let mut data_offset = 0;

                if head_buffer.len() < MAX_HEADER_SIZE {
                    let take = (MAX_HEADER_SIZE - head_buffer.len()).min(data_len);
                    head_buffer.extend_from_slice(&data[..take]);
                    data_offset += take;
                }

                if data_offset < data_len {
                    hasher.update(&data[data_offset..]);
                }

                let progress_percentage = progress * 100 / total;
                if progress_percentage != prev_percentage {
                    let status = format!(
                        "⤒  Converting {}  {:02}%",
                        meta.game_title(),
                        progress_percentage
                    )
                    .to_shared_string();
                    let _ = weak.upgrade_in_event_loop(move |state| state.set_status(status));

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
            head_buffer[..finalization.header.len()].copy_from_slice(&finalization.header);
        }

        split_writer.flush()?;

        let mut final_hasher = Hasher::new();
        final_hasher.update(&head_buffer);
        final_hasher.combine(&hasher);
        let checksum = final_hasher.finalize();
        fs::write(hash_path, format!("{checksum:08x}"))?;

        for path in files_to_remove {
            let _ = fs::remove_file(path);
        }

        Ok(())
    }
}
