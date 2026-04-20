// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow, bail};
use crc32fast::Hasher;
use nod::{
    common::Format,
    read::{DiscOptions, DiscReader, PartitionEncryption},
    write::{DiscWriter, ProcessOptions, ScrubLevel},
};
use slint::{ToSharedString, Weak};
use split_write::SplitWriter;
use zip::ZipArchive;

use crate::{
    ConfigContents, ConversionKind, DriveInfo, GcOutputFormat, QueuedConversion,
    QueuedStandardConversion, Rust, WiiOutputFormat,
    convert::{HEADER_SIZE, SPLIT_SIZE},
    extensions::format_to_opts,
    id_map,
    util::{self, get_threads_num},
};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, BufWriter, Write},
    path::PathBuf,
    time::{Duration, Instant},
};

#[allow(clippy::struct_excessive_bools)]
pub struct StandardConversion {
    pub in_path: PathBuf,
    pub drive_path: PathBuf,
    pub is_wii: bool,
    pub game_title: String,
    pub game_id: String,
    pub disc_number: i32,
    pub always_split: bool,
    pub is_fat32: bool,
    pub wii_output_format: WiiOutputFormat,
    pub gc_output_format: GcOutputFormat,
    pub scrub: bool,
    pub files_to_remove: Vec<PathBuf>,
}

impl StandardConversion {
    pub fn new(
        queued: &QueuedStandardConversion,
        conf: &ConfigContents,
        drive_info: &DriveInfo,
    ) -> Self {
        let mut files_to_remove = Vec::new();
        if conf.remove_sources_games {
            files_to_remove.push(PathBuf::from(&queued.in_path));
        }

        Self {
            in_path: PathBuf::from(&queued.in_path),
            drive_path: PathBuf::from(&conf.mount_point),
            is_wii: queued.is_wii,
            game_title: queued.game_title.to_string(),
            game_id: queued.game_id.to_string(),
            disc_number: queued.disc_number,
            always_split: conf.always_split,
            is_fat32: drive_info.fs_kind == "FAT32",
            wii_output_format: conf.wii_output_format,
            gc_output_format: conf.gc_output_format,
            scrub: conf.scrub_update_partition,
            files_to_remove,
        }
    }

    pub fn perform(&mut self, weak: &Weak<Rust<'static>>) -> Result<()> {
        self.unzip(weak)?;

        let (processor_threads, preloader_threads) = get_threads_num();

        let disc_opts = DiscOptions {
            partition_encryption: PartitionEncryption::Original,
            preloader_threads,
        };

        let should_split = self.is_wii && (self.always_split || self.is_fat32);

        let display_title: &str = id_map::get(&self.game_id).map_or(&self.game_title, |e| e.title);
        let sanitized_title = util::sanitize(display_title);
        let parent_dir_name = if self.is_wii { "wbfs" } else { "games" };
        let game_dir_name = format!("{} [{}]", sanitized_title, &self.game_id);
        let game_dir = self.drive_path.join(parent_dir_name).join(game_dir_name);

        let get_file_name = |i| {
            if self.is_wii {
                match self.wii_output_format {
                    WiiOutputFormat::Iso => {
                        if should_split {
                            format!("{}.part{i}.iso", &self.game_id)
                        } else {
                            format!("{}.iso", &self.game_id)
                        }
                    }
                    WiiOutputFormat::Wbfs => match i {
                        0 => format!("{}.wbfs", &self.game_id),
                        n => format!("{}.wbf{n}", &self.game_id),
                    },
                }
            } else {
                match self.gc_output_format {
                    GcOutputFormat::Iso => match self.disc_number {
                        0 => "game.iso".to_string(),
                        n => format!("disc{}.iso", n + 1),
                    },

                    GcOutputFormat::Ciso => match self.disc_number {
                        0 => "game.ciso".to_string(),
                        n => format!("disc{}.ciso", n + 1),
                    },
                }
            }
        };

        let out_format = match (self.is_wii, self.wii_output_format, self.gc_output_format) {
            (true, WiiOutputFormat::Iso, _) | (false, _, GcOutputFormat::Iso) => Format::Iso,
            (true, WiiOutputFormat::Wbfs, _) => Format::Wbfs,
            (false, _, GcOutputFormat::Ciso) => Format::Ciso,
        };

        let scrub = if self.scrub {
            ScrubLevel::UpdatePartition
        } else {
            ScrubLevel::None
        };

        let out_opts = format_to_opts(out_format);
        let process_opts = ProcessOptions {
            processor_threads,
            scrub,
            digest_crc32: false,
            digest_md5: false,
            digest_sha1: false,
            digest_xxh64: false,
        };

        let split_size = if should_split { Some(SPLIT_SIZE) } else { None };

        let hash_path = game_dir.join(format!("{}.crc32", &self.game_id));

        let mut out_writer = BufWriter::with_capacity(
            32_768,
            SplitWriter::create(&game_dir, get_file_name, split_size)?,
        );

        let disc_reader = DiscReader::new(&self.in_path, &disc_opts)?;
        let disc_writer = DiscWriter::new(disc_reader, &out_opts)?;
        let mut head_buffer = Vec::with_capacity(HEADER_SIZE);
        let mut hasher = Hasher::new();

        let mut last_update = Instant::now();
        let finalization = disc_writer.process(
            |data, progress, total| {
                out_writer.write_all(&data)?;

                let remaining_in_head = HEADER_SIZE.saturating_sub(head_buffer.len());
                if remaining_in_head > 0 {
                    let to_write = remaining_in_head.min(data.len());
                    head_buffer.extend_from_slice(&data[..to_write]);
                    hasher.update(&data[to_write..]);
                } else {
                    hasher.update(&data);
                }

                if last_update.elapsed() > Duration::from_millis(100) {
                    let current_percentage = progress * 100 / total;

                    let status = format!(
                        "⤒  Converting {}  {:02}%",
                        &self.game_title, current_percentage
                    );
                    let _ = weak.upgrade_in_event_loop(move |rust| {
                        rust.set_status(status.to_shared_string());
                    });

                    last_update = Instant::now();
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
        drop(split_writer);
        drop(disc_writer);

        let mut final_hasher = Hasher::new();
        final_hasher.update(&head_buffer);
        final_hasher.combine(&hasher);
        let checksum = final_hasher.finalize();
        fs::write(hash_path, format!("{checksum:08x}"))?;

        for path in &self.files_to_remove {
            let _ = fs::remove_file(path);
        }

        Ok(())
    }

    fn unzip(&mut self, weak: &Weak<Rust<'static>>) -> Result<()> {
        let is_zip = self
            .in_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"));

        if !is_zip {
            return Ok(());
        }

        let status = format!("Unzipping {}", self.in_path.display());
        let _ = weak.upgrade_in_event_loop(move |rust| {
            rust.set_status(status.to_shared_string());
        });

        let mut f = File::open(&self.in_path)?;
        let mut archive = ZipArchive::new(&mut f)?;
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

pub fn make_queue(paths: Vec<PathBuf>, existing_ids: &[String]) -> Vec<QueuedConversion> {
    // parse discs
    let mut entries = paths
        .into_iter()
        .filter_map(|p| {
            let mut f = File::open(&p).ok()?;

            let meta = if p
                .extension()
                .and_then(OsStr::to_str)
                .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
            {
                let mut archive = ZipArchive::new(&mut f).ok()?;
                let mut disc = archive.by_index(0).ok()?;
                wii_disc_info::Meta::read(&mut disc).ok()?
            } else {
                wii_disc_info::Meta::read(&mut f).ok()?
            };

            Some((p, meta))
        })
        .collect::<Vec<_>>();

    // keep only new games
    entries.retain(|(_, meta)| existing_ids.iter().all(|id| id != meta.game_id()));

    let mut queue = Vec::new();
    for (path, meta) in entries {
        let queued = QueuedStandardConversion {
            game_title: meta.game_title().to_shared_string(),
            game_id: meta.game_id().to_shared_string(),
            in_path: path.to_string_lossy().to_shared_string(),
            is_wii: meta.is_wii(),
            disc_number: i32::from(meta.disc_number()),
        };

        let queued = QueuedConversion {
            kind: ConversionKind::Standard,
            standard: queued,
            ..Default::default()
        };

        queue.push(queued);
    }

    queue
}
