// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize)]
pub struct GameEntry {
    id: [u8; 6],
    pub ghid: Option<u32>,
    pub title: String,
}

#[derive(Deserialize)]
pub struct IdMap(Box<[GameEntry]>);

impl IdMap {
    pub fn get(&self, id: &str) -> Option<&GameEntry> {
        let id = id.as_bytes();
        let mut buf = [0; 6];
        buf[..id.len()].copy_from_slice(id);

        match self.0.binary_search_by_key(&id, |entry| &entry.id) {
            Ok(i) => Some(&self.0[i]),
            Err(_) => None,
        }
    }
}

pub static ID_MAP: LazyLock<IdMap> = LazyLock::new(|| {
    const RAW: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));
    postcard::from_bytes(RAW).unwrap()
});
