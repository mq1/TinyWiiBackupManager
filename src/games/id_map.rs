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

const COMPRESSED_ID_MAP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin.zst"));

static ID_MAP: LazyLock<Vec<GameEntry>> = LazyLock::new(|| {
    let bytes = zstd::bulk::decompress(COMPRESSED_ID_MAP, DATA_SIZE).unwrap();
    postcard::from_bytes(&bytes).unwrap()
});

pub fn get_title(game_id: GameID) -> Option<&'static str> {
    let i = ID_MAP
        .binary_search_by_key(&game_id, |entry| entry.id)
        .ok()?;

    Some(&ID_MAP[i].title)
}

pub fn get_ghid(game_id: GameID) -> Option<u32> {
    let i = ID_MAP
        .binary_search_by_key(&game_id, |entry| entry.id)
        .ok()?;

    match ID_MAP[i].ghid {
        0 => None,
        ghid => Some(ghid),
    }
}

pub fn get_init_task() -> Task<Message> {
    Task::perform(async { LazyLock::force(&ID_MAP) }, |_| Message::None).discard()
}
