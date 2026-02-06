// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// Handy functions for when using nod could be overkill

use crate::games::game_id::GameID;
use nod::common::Format;
use std::io::{self, Read};

fn get_gameid_offset(format: Format) -> Option<u64> {
    match format {
        Format::Iso => Some(0),
        Format::Wbfs => Some(512),
        Format::Rvz | Format::Wia => Some(88),
        Format::Ciso => Some(32768),
        _ => None,
    }
}

pub fn read_gameid<R: Read>(reader: &mut R, format: Format) -> Option<GameID> {
    let offset = get_gameid_offset(format)?;

    // skip to id position
    if offset > 0 {
        io::copy(&mut reader.take(offset), &mut io::sink()).ok()?;
    }

    // read and parse the id
    let mut id_buf = [0u8; 6];
    reader.read_exact(&mut id_buf).ok()?;
    let id = GameID::from(id_buf);

    Some(id)
}
