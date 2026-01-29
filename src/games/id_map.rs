// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;
use crate::message::Message;
use iced::Task;
use std::str;
use std::sync::LazyLock;

const COMPRESSED_ID_MAP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin.zst"));

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

pub struct IdMap {
    ids: Vec<GameID>,
    ghids: Vec<u32>,
    titles: Vec<&'static str>,
}

impl IdMap {
    pub fn get_title(&self, game_id: GameID) -> Option<&str> {
        self.ids
            .binary_search(&game_id)
            .ok()
            .map(|i| self.titles[i])
    }

    pub fn get_ghid(&self, game_id: GameID) -> Option<u32> {
        self.ids.binary_search(&game_id).ok().map(|i| self.ghids[i])
    }
}

pub static ID_MAP: LazyLock<IdMap> = LazyLock::new(deserialize_id_map);

fn deserialize_id_map() -> IdMap {
    // Decompress
    let serialized_id_map = zstd::bulk::decompress(COMPRESSED_ID_MAP, ID_MAP_BYTES_LEN)
        .expect("Failed to decompress ID map");

    // Leak the buffer to promote it to &'static [u8]
    // This allows us to store &str references without allocating new Strings.
    let input: &'static [u8] = Box::leak(serialized_id_map.into_boxed_slice());

    let mut ids = Vec::with_capacity(ID_COUNT);
    let mut ghids = Vec::with_capacity(ID_COUNT);
    let mut titles = Vec::with_capacity(ID_COUNT);

    let mut cursor = 0;
    while cursor < input.len() {
        // Parse game id (6 bytes)
        let gid_slice = &input[cursor..cursor + 6];
        let game_id: [u8; 6] = gid_slice.try_into().unwrap();
        ids.push(game_id.into());
        cursor += 6;

        // Parse gamehacking id (4 bytes)
        let gh_slice = &input[cursor..cursor + 4];
        let gamehacking_id = u32::from_le_bytes(gh_slice.try_into().unwrap());
        ghids.push(gamehacking_id);
        cursor += 4;

        // Parse title len
        let str_len = input[cursor] as usize;
        cursor += 1;

        // Parse title string
        let title_slice = &input[cursor..cursor + str_len];
        let title = unsafe { str::from_utf8_unchecked(title_slice) };
        titles.push(title);

        cursor += str_len;
    }

    IdMap { ids, ghids, titles }
}

pub fn get_init_task() -> Task<Message> {
    Task::perform(
        async {
            LazyLock::force(&ID_MAP);
        },
        |()| Message::EmptyResult(Ok(())),
    )
}
