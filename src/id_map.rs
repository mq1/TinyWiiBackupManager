// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::num::NonZeroU32;

pub struct GameEntry {
    id: [u8; 6],
    pub ghid: Option<NonZeroU32>,
    pub title: &'static str,
}

#[cfg(not(feature = "rust-analyzer"))]
const fn g(id: [u8; 6], ghid: Option<u32>, title: &'static str) -> GameEntry {
    match ghid {
        Some(ghid) => GameEntry {
            id,
            ghid: Some(unsafe { NonZeroU32::new_unchecked(ghid) }),
            title,
        },
        None => GameEntry {
            id,
            ghid: None,
            title,
        },
    }
}

#[cfg(not(feature = "rust-analyzer"))]
include!(concat!(env!("OUT_DIR"), "/id_map_generated.rs"));

#[cfg(feature = "rust-analyzer")]
const GAMES: &[GameEntry] = &[];

pub fn get(id: &str) -> Option<&'static GameEntry> {
    let mut buf = [0; 6];
    let bytes = id.as_bytes();
    let len = bytes.len().min(6);
    buf[..len].copy_from_slice(&bytes[..len]);

    let i = GAMES.binary_search_by_key(&buf, |e| e.id).ok()?;
    let entry = &GAMES[i];

    Some(entry)
}
