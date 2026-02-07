// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// Handy functions for when using nod could be overkill

use crate::games::game_id::GameID;
use nod::common::Format;
use std::io::{self, Read};

fn get_gameid_offset(format: Format) -> Option<u64> {
    match format {
        Format::Iso | Format::Gcz => Some(0),
        Format::Wbfs => Some(512),
        Format::Rvz | Format::Wia => Some(88),
        Format::Ciso | Format::Tgc => Some(32768),
        Format::Nfs => None,
    }
}

fn get_title_offset(format: Format) -> Option<u64> {
    match format {
        Format::Iso | Format::Gcz => Some(32),
        Format::Wbfs => Some(544),
        Format::Rvz | Format::Wia => Some(120),
        Format::Ciso | Format::Tgc => Some(32800),
        Format::Nfs => None,
    }
}

pub fn read_gameid<R: Read>(reader: &mut R, format: Format) -> Option<GameID> {
    read_gameid_and_title(reader, format).map(|(id, _)| id)
}

pub fn read_gameid_and_title<R: Read>(reader: &mut R, format: Format) -> Option<(GameID, String)> {
    let id_offset = get_gameid_offset(format)?;
    let title_offset = get_title_offset(format)?;

    // skip to id position
    if id_offset > 0 {
        io::copy(&mut reader.take(id_offset), &mut io::sink()).ok()?;
    }

    // read and parse the id
    let mut id_buf = [0u8; 6];
    reader.read_exact(&mut id_buf).ok()?;
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

    Some((id, title))
}
