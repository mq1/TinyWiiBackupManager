// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::extensions::SUPPORTED_INPUT_EXTENSIONS;
use crate::{convert::get_disc_opts, overflow_reader::get_main_file};
use anyhow::{Result, anyhow, bail};
use nod::{
    disc::DiscHeader,
    read::{DiscMeta, DiscReader},
};
use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug, Clone)]
pub struct DiscInfo {
    pub main_disc_path: PathBuf,
    pub header: DiscHeader,
    pub meta: DiscMeta,
}

impl DiscInfo {
    pub fn from_game_dir(game_dir: &Path) -> Result<DiscInfo> {
        let main_disc_path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
        let disc = DiscReader::new(&main_disc_path, &get_disc_opts())?;

        let header = disc.header().to_owned();
        let meta = disc.meta();

        Ok(Self {
            main_disc_path,
            header,
            meta,
        })
    }

    pub fn from_main_file(main_disc_path: PathBuf) -> Result<DiscInfo> {
        let disc = if main_disc_path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ["zip", "ZIP"].contains(&ext))
        {
            let file_reader = BufReader::new(File::open(&main_disc_path)?);
            let mut archive = ZipArchive::new(file_reader)?;

            // for now, only the first file is read
            let mut disc_file = archive.by_index(0)?;

            if !SUPPORTED_INPUT_EXTENSIONS
                .iter()
                .any(|ext| disc_file.name().ends_with(ext))
            {
                bail!("Skipping invalid zip");
            }

            // for now we read 1 MB, TODO check the actual size of the header
            let mut buf = vec![0u8; 1024 * 1024].into_boxed_slice();
            disc_file.read_exact(&mut buf)?;
            let cursor = Cursor::new(buf);

            DiscReader::new_from_non_cloneable_read(cursor, &get_disc_opts())?
        } else {
            DiscReader::new(&main_disc_path, &get_disc_opts())?
        };

        Ok(Self {
            main_disc_path,
            header: disc.header().clone(),
            meta: disc.meta(),
        })
    }
}
