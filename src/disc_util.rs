// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// Handy functions for when using nod could be overkill

use crate::games::game_id::GameID;
use anyhow::Result;
use nod::common::Format;
use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

fn get_gameid_offset(format: Format) -> Option<u64> {
    match format {
        Format::Iso => Some(0),
        Format::Wbfs => Some(512),
        Format::Rvz | Format::Wia => Some(88),
        Format::Ciso | Format::Tgc => Some(32768),
        _ => None,
    }
}

fn get_title_offset(format: Format) -> Option<u64> {
    match format {
        Format::Iso => Some(32),
        Format::Wbfs => Some(544),
        Format::Rvz | Format::Wia => Some(120),
        Format::Ciso | Format::Tgc => Some(32800),
        _ => None,
    }
}

pub fn read_disc_header<R: Read>(reader: &mut R) -> Option<(Format, GameID, String)> {
    let (format, mut id_buf) = guess_format(reader).ok()?;
    let id_offset = get_gameid_offset(format)?;
    let title_offset = get_title_offset(format)?;

    // read and parse the id
    if format != Format::Iso {
        // skip to id position
        io::copy(&mut reader.take(id_offset - 6), &mut io::sink()).ok()?;
        reader.read_exact(&mut id_buf).ok()?;
    }

    let id = GameID::from(id_buf);
    eprintln!("Parsed GameID: {}", id.as_str());

    // skip to the title offset
    let bytes_to_skip = title_offset - id_offset - 6;
    io::copy(&mut reader.take(bytes_to_skip), &mut io::sink()).ok()?;

    // read and parse the title
    let mut title_buf = [0u8; 64];
    reader.read_exact(&mut title_buf).ok()?;
    let title = String::from_utf8_lossy(&title_buf).to_string();
    eprintln!("Parsed Title: {title}");

    Some((format, id, title))
}

pub fn guess_format<R: Read>(reader: &mut R) -> Result<(Format, [u8; 6])> {
    let mut id_buf = [0u8; 6];
    reader.read_exact(&mut id_buf)?;

    let format = match id_buf[0..4] {
        [b'R', b'V', b'Z', 0x01] => Format::Rvz,
        [b'W', b'I', b'A', 0x01] => Format::Wia,
        [b'C', b'I', b'S', b'O'] => Format::Ciso,
        [b'W', b'B', b'F', b'S'] => Format::Wbfs,
        [0x01, 0xC0, 0x0B, 0xB1] => Format::Gcz,
        [b'E', b'G', b'G', b'S'] => Format::Nfs,
        [0xAE, 0x0F, 0x38, 0xA2] => Format::Tgc,
        _ => Format::Iso,
    };

    Ok((format, id_buf))
}

pub fn read_disc_header_from_game_dir(path: &Path) -> Option<(Format, GameID, String)> {
    for entry in fs::read_dir(path).ok()?.filter_map(Result::ok) {
        let Ok(mut file) = File::open(entry.path()) else {
            continue;
        };
        if let Some((format, id, title)) = read_disc_header(&mut file) {
            return Some((format, id, title));
        }
    }

    None
}
