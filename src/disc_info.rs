// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{DiscInfo, convert::get_disc_opts, overflow_reader::get_main_file};
use anyhow::{Result, anyhow};
use nod::read::DiscReader;
use size::Size;
use slint::ToSharedString;
use std::path::Path;

pub fn get_region_display(id: [u8; 6]) -> &'static str {
    match id[3] {
        b'A' => "System Wii Channels (i.e. Mii Channel)",
        b'B' => "Ufouria: The Saga (NA)",
        b'D' => "Germany",
        b'E' => "USA",
        b'F' => "France",
        b'H' => "Netherlands / Europe alternate languages",
        b'I' => "Italy",
        b'J' => "Japan",
        b'K' => "Korea",
        b'L' => "Japanese import to Europe, Australia and other PAL regions",
        b'M' => "American import to Europe, Australia and other PAL regions",
        b'N' => "Japanese import to USA and other NTSC regions",
        b'P' => "Europe and other PAL regions such as Australia",
        b'Q' => "Japanese Virtual Console import to Korea",
        b'R' => "Russia",
        b'S' => "Spain",
        b'T' => "American Virtual Console import to Korea",
        b'U' => "Australia / Europe alternate languages",
        b'V' => "Scandinavia",
        b'W' => "Republic of China (Taiwan) / Hong Kong / Macau",
        b'X' => "Europe alternate languages / US special releases",
        b'Y' => "Europe alternate languages / US special releases",
        b'Z' => "Europe alternate languages / US special releases",
        _ => "Unknown",
    }
}

pub fn get_disc_info(game_dir_str: &str) -> Result<DiscInfo> {
    let game_dir = Path::new(game_dir_str);

    let path = get_main_file(game_dir).ok_or(anyhow!("No disc found"))?;
    let game_dir = game_dir
        .to_str()
        .ok_or(anyhow!("Invalid path"))?
        .to_shared_string();

    let disc = DiscReader::new(&path, &get_disc_opts())?;

    // Header
    let header = disc.header();
    let game_id = header.game_id_str().to_shared_string();
    let game_title = header.game_title_str().to_shared_string();
    let is_gamecube = header.is_gamecube();
    let is_wii = header.is_wii();
    let disc_num = header.disc_num as i32;
    let disc_version = header.disc_version as i32;
    let region = get_region_display(header.game_id).to_shared_string();

    // Meta
    let meta = disc.meta();
    let format = meta.format.to_shared_string();
    let compression = meta.compression.to_shared_string();
    let block_size = meta
        .block_size
        .map(|bs| Size::from_bytes(bs).to_shared_string())
        .unwrap_or("Unknown".to_shared_string());
    let decrypted = meta.decrypted;
    let needs_hash_recovery = meta.needs_hash_recovery;
    let lossless = meta.lossless;
    let disc_size = meta
        .disc_size
        .map(|ds| Size::from_bytes(ds).to_shared_string())
        .unwrap_or("Unknown".to_shared_string());
    let crc32 = meta
        .crc32
        .map(|hash| format!("{:08x}", hash).to_shared_string())
        .unwrap_or("Unknown".to_shared_string());
    let md5 = meta
        .md5
        .map(|hash| hex::encode(hash).to_shared_string())
        .unwrap_or("Unknown".to_shared_string());
    let sha1 = meta
        .sha1
        .map(|hash| hex::encode(hash).to_shared_string())
        .unwrap_or("Unknown".to_shared_string());
    let xxh64 = meta
        .xxh64
        .map(|hash| format!("{:08x}", hash).to_shared_string())
        .unwrap_or("Unknown".to_shared_string());

    Ok(DiscInfo {
        game_dir,
        game_id,
        game_title,
        is_gamecube,
        is_wii,
        disc_num,
        disc_version,
        region,
        format,
        compression,
        block_size,
        decrypted,
        needs_hash_recovery,
        lossless,
        disc_size,
        crc32,
        md5,
        sha1,
        xxh64,
    })
}
