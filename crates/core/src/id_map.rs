// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::num::NonZeroU32;
use crate::game_id::GameID;

pub struct GameEntry {
    id: GameID,
    pub ghid: Option<NonZeroU32>,
    pub title: &'static str,
}

#[cfg(not(feature = "rust-analyzer"))]
const fn g(id: u32, ghid: Option<u32>, title: &'static str) -> GameEntry {
    match ghid {
        Some(ghid) => GameEntry {
            id: GameID(id),
            ghid: Some(unsafe { NonZeroU32::new_unchecked(ghid) }),
            title,
        },
        None => GameEntry {
            id: GameID(id),
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
    let id = GameID::new(id)?;
    let i = GAMES.binary_search_by_key(&id, |e| e.id).ok()?;
    let entry = &GAMES[i];

    Some(entry)
}
