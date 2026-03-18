// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;
use crate::message::Message;
use derive_getters::Getters;
use iced::Task;
use serde::Deserialize;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[derive(Deserialize, Getters)]
pub struct GameEntry {
    #[getter(copy)]
    id: GameID,
    #[getter(copy)]
    ghid: Option<u32>,
    title: String,
}

#[derive(Deserialize)]
pub struct IdMap(Box<[GameEntry]>);

impl IdMap {
    pub fn get(&self, id: GameID) -> Option<&GameEntry> {
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

pub fn get_init_task() -> Task<Message> {
    Task::perform(async { LazyLock::force(&ID_MAP) }, |_| Message::None).discard()
}
