// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[derive(Debug, Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Deserialize)]
struct GameID([u8; 6]);

impl From<[u8; 6]> for GameID {
    fn from(id: [u8; 6]) -> Self {
        Self(id)
    }
}

impl From<&str> for GameID {
    fn from(id: &str) -> Self {
        let bytes = id.as_bytes();
        let mut buf = [0; 6];
        buf[..bytes.len()].copy_from_slice(bytes);
        Self(buf)
    }
}

#[derive(Deserialize)]
pub struct GameEntry {
    id: GameID,
    pub ghid: Option<u32>,
    pub title: String,
}

#[derive(Deserialize)]
pub struct IdMap(Box<[GameEntry]>);

impl IdMap {
    pub fn get(&self, id: impl Into<GameID>) -> Option<&GameEntry> {
        let id = id.into();

        match self.0.binary_search_by_key(&id, |entry| entry.id) {
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
