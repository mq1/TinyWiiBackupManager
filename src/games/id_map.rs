// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;
use crate::message::Message;
use iced::Task;
use serde::Deserialize;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[derive(Deserialize)]
pub struct GameEntry {
    id: GameID,
    pub ghid: Option<u32>,
    pub title: String,
}

#[derive(Deserialize)]
pub struct IdMap(Box<[GameEntry]>);

impl IdMap {
    pub fn get(&self, id: GameID) -> Option<&GameEntry> {
        self.0
            .binary_search_by_key(&id, |entry| entry.id)
            .ok()
            .map(|i| &self.0[i])
    }
}

pub static ID_MAP: LazyLock<IdMap> = LazyLock::new(|| {
    const COMPRESSED_ID_MAP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin.zst"));
    let bytes = zstd::bulk::decompress(COMPRESSED_ID_MAP, DATA_SIZE).unwrap();
    postcard::from_bytes(&bytes).unwrap()
});

pub fn get_init_task() -> Task<Message> {
    Task::perform(async { LazyLock::force(&ID_MAP) }, |_| Message::None).discard()
}
