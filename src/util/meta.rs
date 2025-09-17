// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::util::fs::find_disc;
use anyhow::{Result, bail};
use nod::read::DiscMeta;
use nod::read::{DiscOptions, DiscReader};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Reads the MD5 from the .wbfs file at 0x2EC
///
/// This is for Wii Backup Manager compatibility
/// Draft, I don't know how to verify the wbm md5 yet
#[allow(dead_code)]
fn fallback_md5(path: impl AsRef<Path>) -> Result<[u8; 16]> {
    let mut file = File::open(path)?;

    // Verify if 0x2EC is "MD5#" (0x4d 0x44 0x35 0x23)
    file.seek(SeekFrom::Start(0x2EC))?;
    let mut magic = [0; 4];
    file.read_exact(&mut magic)?;
    if magic != [0x4d, 0x44, 0x35, 0x23] {
        bail!("Invalid md5 magic");
    }

    // Read the MD5
    let mut md5 = [0; 16];
    file.read_exact(&mut md5)?;

    Ok(md5)
}

pub fn read_meta(game_dir: impl AsRef<Path>) -> Result<DiscMeta> {
    let path = find_disc(game_dir)?;
    let reader = DiscReader::new(&path, &DiscOptions::default())?;

    #[allow(unused_mut)]
    let mut meta = reader.meta();

    //if meta.md5.is_none() && let Ok(md5) = fallback_md5(&path) {
    //    meta.md5 = Some(md5);
    //}

    Ok(meta)
}
