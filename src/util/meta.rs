// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::Game;
use crate::util::fs::find_disc;
use anyhow::Result;
use nod::common::Format;
use nod::disc::DiscHeader;
use nod::read::DiscMeta;
use nod::read::{DiscOptions, DiscReader};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

fn fallback_md5(path: impl AsRef<Path>) -> Result<[u8; 16]> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(0x2EC))?;
    let mut buffer = [0; 16];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

pub fn read_header_and_meta(game: &Game) -> Result<(DiscHeader, DiscMeta)> {
    let path = find_disc(game)?;
    let reader = DiscReader::new(&path, &DiscOptions::default())?;

    let header = reader.header().clone();
    let mut meta = reader.meta();

    // If the .wbfs file was created by Wii Backup Manager, the MD5 is stored at 0x2EC
    if meta.md5.is_none() && meta.format == Format::Wbfs && !meta.lossless {
        meta.md5 = fallback_md5(path).ok()
    }

    Ok((header, meta))
}
