// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::GameID;
use derive_getters::Getters;
use serde::Deserialize;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[derive(Deserialize, Getters)]
pub struct GameEntry {
    #[getter(skip)]
    id: [u8; 6],
    ghid: Option<u32>,
    title: String,
}

#[derive(Deserialize)]
pub struct IdMap(Box<[GameEntry]>);

impl IdMap {
    pub fn get(&self, id: &GameID) -> Option<&GameEntry> {
        let raw = id.as_raw();

        match self.0.binary_search_by_key(&raw, |entry| entry.id) {
            Ok(i) => Some(&self.0[i]),
            Err(_) => None,
        }
    }
}

pub static ID_MAP: LazyLock<IdMap> = LazyLock::new(|| {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

    #[cfg(feature = "compress-idmap")]
    let bytes = &zstd::bulk::decompress(bytes, UNCOMPRESSED_SIZE).unwrap();

    postcard::from_bytes(bytes).unwrap()
});
