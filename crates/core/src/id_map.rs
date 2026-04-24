// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game_id::GameID;
use std::num::NonZeroU32;

pub struct GameEntry {
    id: GameID,
    pub ghid: Option<NonZeroU32>,
    pub title: &'static str,
}

impl GameEntry {
    pub const fn new(id: u32, ghid: u32, title: &'static str) -> Self {
        GameEntry {
            id: GameID::from_u32(id),
            ghid: NonZeroU32::new(ghid),
            title,
        }
    }
}

#[cfg(not(feature = "rust-analyzer"))]
include!(concat!(env!("OUT_DIR"), "/id_map_generated.rs"));

#[cfg(feature = "rust-analyzer")]
const GAMES: &[GameEntry] = &[];

pub fn get(id: GameID) -> Option<&'static GameEntry> {
    let i = GAMES.binary_search_by_key(&id, |e| e.id).ok()?;
    let entry = &GAMES[i];

    Some(entry)
}
