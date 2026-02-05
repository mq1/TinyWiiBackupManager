// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// Handy functions for when using nod could be overkill

use crate::games::game_id::GameID;
use nod::common::Format;
use std::io::{self, Read};

pub fn read_gameid<R: Read>(reader: &mut R, format: Format) -> Option<GameID> {
    let id_pos = match format {
        Format::Iso => 0,
        Format::Wbfs => 512,
        Format::Rvz => 88,
        Format::Ciso => 32768,
        _ => {
            return None;
        }
    };

    // skip to id position
    if id_pos > 0 {
        io::copy(&mut reader.take(id_pos), &mut io::sink()).ok()?;
    }

    // read and parse the id
    let mut id_buf = [0u8; 6];
    reader.read_exact(&mut id_buf).ok()?;
    let id = GameID::from(id_buf);

    Some(id)
}
