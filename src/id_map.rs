// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

include!(concat!(env!("OUT_DIR"), "/gamehacking_ids.rs"));

pub fn get_gamehacking_id(id: [u8; 6]) -> Option<u32> {
    GAMEID_TO_GHID
        .binary_search_by_key(&id, |entry| entry.0)
        .map(|i| GAMEID_TO_GHID[i].1)
        .ok()
}
