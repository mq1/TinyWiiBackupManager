// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;

include!(concat!(env!("OUT_DIR"), "/gamehacking_ids.rs"));

pub fn get_gamehacking_id(game_id: GameID) -> Option<u32> {
    let raw_id: [u8; 6] = game_id.into();

    GAMEID_TO_GHID
        .binary_search_by_key(&raw_id, |entry| entry.0)
        .map(|i| GAMEID_TO_GHID[i].1)
        .ok()
}
