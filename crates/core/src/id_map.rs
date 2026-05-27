// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game_id::GameID;
use rkyv::{Archive, Deserialize, vec::ArchivedVec};
use std::num::NonZeroU32;

const BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

#[derive(Archive, Deserialize)]
pub struct GameEntry {
    id: u32,
    pub ghid: Option<NonZeroU32>,
    pub title: String,
}

pub fn get(id: GameID) -> Option<&'static ArchivedGameEntry> {
    let archived = unsafe { rkyv::access_unchecked::<ArchivedVec<ArchivedGameEntry>>(BYTES) };

    let id = id.to_u32().into();
    match archived.binary_search_by_key(&id, |e| e.id) {
        Ok(idx) => Some(&archived[idx]),
        Err(_) => None,
    }
}
