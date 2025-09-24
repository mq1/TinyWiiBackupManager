// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::AGENT;
use crate::base_dir::BaseDir;
use anyhow::{Context, Result};
use rkyv::boxed::ArchivedBox;
use rkyv::{Archive, Deserialize, rancor};
use std::fs::{self, File};
use std::io;
use std::io::Cursor;
use std::sync::LazyLock;
use strum_macros::{Display, EnumMessage, IntoStaticStr};

const DOWNLOAD_URL: &str = "https://www.gametdb.com/wiitdb.zip";
include!(concat!(env!("OUT_DIR"), "/metadata.rs"));

pub static WIITDB: LazyLock<Box<[u8; WIITDB_SIZE]>> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/wiitdb.bin.zst"));
    let decompressed = zstd::bulk::decompress(bytes, WIITDB_SIZE).expect("failed to decompress");
    let boxed = decompressed.into_boxed_slice();
    boxed.try_into().expect("failed to convert to array")
});

pub fn lookup(id: &[u8; 6]) -> Option<GameInfo> {
    // unsafe but we deserialize known data, so it's safe
    let archived = unsafe {
        rkyv::access_unchecked::<ArchivedBox<[([u8; 6], ArchivedGameInfo)]>>(&WIITDB[..])
    };
    let index = archived.binary_search_by_key(id, |game| game.0).ok()?;
    rkyv::deserialize::<_, rancor::Error>(&archived[index].1).ok()
}

#[derive(Deserialize, Archive, Debug, Clone, Copy, IntoStaticStr)]
pub enum Language {
    #[strum(serialize = "English")]
    En,
    #[strum(serialize = "French")]
    Fr,
    #[strum(serialize = "German")]
    De,
    #[strum(serialize = "Spanish")]
    Es,
    #[strum(serialize = "Italian")]
    It,
    #[strum(serialize = "Japanese")]
    Ja,
    #[strum(serialize = "Dutch")]
    Nl,
    #[strum(serialize = "Swedish")]
    Se,
    #[strum(serialize = "Danish")]
    Dk,
    #[strum(serialize = "Norwegian")]
    No,
    #[strum(serialize = "Korean")]
    Ko,
    #[strum(serialize = "Portuguese")]
    Pt,
    #[strum(serialize = "Mandarin (Taiwan)")]
    Zhtw,
    #[strum(serialize = "Mandarin (China)")]
    Zhcn,
    #[strum(serialize = "Finnish")]
    Fi,
    #[strum(serialize = "Turkish")]
    Tr,
    #[strum(serialize = "Greek")]
    Gr,
    #[strum(serialize = "Russian")]
    Ru,
}

#[derive(Deserialize, Archive, Debug, Clone, Copy, IntoStaticStr, Display, EnumMessage)]
pub enum Region {
    #[strum(serialize = "NTSC-J (Japan)", message = "JA")]
    NtscJ,
    #[strum(serialize = "NTSC-U (USA)", message = "US")]
    NtscU,
    #[strum(serialize = "NTSC-K (South Korea)", message = "KO")]
    NtscK,
    #[strum(serialize = "NTSC-T (Taiwan)", message = "ZH")]
    NtscT,
    #[strum(serialize = "PAL (Europe)", message = "EN")]
    Pal,
    #[strum(serialize = "PAL-R (Russia)", message = "RU")]
    PalR,
}

/// Data from WiiTDB XML
#[derive(Deserialize, Archive, Debug, Clone)]
pub struct GameInfo {
    pub title: String,
    pub region: Region,
    pub languages: Vec<Language>,
    pub crc_list: Vec<u32>,
}

/// Handles the blocking logic of downloading and extracting the database.
pub fn download_and_extract_database(base_dir: &BaseDir) -> Result<()> {
    // Create the target directory.
    let target_dir = base_dir.usbloadergx_dir();
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("Failed to create directory at: {}", target_dir.display()))?;

    // Perform the download request.
    let mut response = AGENT
        .get(DOWNLOAD_URL)
        .call()
        .with_context(|| format!("Failed to download from {DOWNLOAD_URL}"))?;

    let buffer = response.body_mut().read_to_vec()?;

    // Create a cursor in memory.
    let cursor = Cursor::new(buffer);

    // Open the zip archive from the in-memory buffer.
    let mut archive =
        zip::ZipArchive::new(cursor).context("Failed to create zip archive from cursor")?;

    let mut zipped_file = archive
        .by_name("wiitdb.xml")
        .context("Could not find 'wiitdb.xml' in the downloaded archive")?;

    // Extract the wiitdb.xml file to the target directory.
    let target_path = target_dir.join("wiitdb.xml");
    let mut outfile = File::create(&target_path)
        .with_context(|| format!("Failed to create output file at: {}", target_path.display()))?;

    io::copy(&mut zipped_file, &mut outfile).with_context(|| {
        format!(
            "Failed to extract 'wiitdb.xml' to {}",
            target_path.display()
        )
    })?;

    Ok(())
}
