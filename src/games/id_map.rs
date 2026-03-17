// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;
use crate::message::Message;
use iced::Task;
use serde::Deserialize;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

#[derive(Deserialize)]
struct GameEntry {
    id: GameID,
    ghid: u32,
    title: String,
}

#[derive(Deserialize)]
pub struct IdMap(Box<[GameEntry]>);

impl<'a> IdMap {
    pub fn get_title(&'a self, id: GameID) -> Option<&'a str> {
        let i = self.0.binary_search_by_key(&id, |entry| entry.id).ok()?;
        let entry = &self.0[i];

        Some(&entry.title)
    }

    pub fn get_ghid(&self, id: GameID) -> Option<u32> {
        let i = self.0.binary_search_by_key(&id, |entry| entry.id).ok()?;
        let entry = &self.0[i];

        match entry.ghid {
            0 => None,
            ghid => Some(ghid),
        }
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
