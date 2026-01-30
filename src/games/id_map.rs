// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;
use crate::message::Message;
use iced::Task;
use std::mem::MaybeUninit;
use std::str;
use std::sync::LazyLock;

const COMPRESSED_ID_MAP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin.zst"));

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

fn u24_le_to_u32(buf: [u8; 3]) -> u32 {
    u32::from_le_bytes([buf[0], buf[1], buf[2], 0])
}

pub struct IdMap {
    ids: [GameID; ID_COUNT],
    ghids: Box<[u32]>,
    titles: Box<[&'static str]>,
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

/// lots of unsafe, we assume the input data is correct (it always is)
fn deserialize_id_map() -> IdMap {
    // Decompress
    let serialized_id_map = zstd::bulk::decompress(COMPRESSED_ID_MAP, ID_MAP_BYTES_LEN)
        .expect("Failed to decompress ID map")
        .into_boxed_slice();

    // Leak the buffer to promote it to &'static [u8]
    // This allows us to store &str references without allocating new Strings.
    let input: &'static [u8] = Box::leak(serialized_id_map);

    let mut ids = MaybeUninit::<[GameID; ID_COUNT]>::uninit();
    let mut ghids = Box::<[u32]>::new_uninit_slice(ID_COUNT);
    let mut titles = Box::<[&'static str]>::new_uninit_slice(ID_COUNT);

    let mut cursor = 0;
    let mut i = 0;

    while i < ID_COUNT {
        // Parse game id (6 bytes)
        let gid_slice = &input[cursor..cursor + 6];
        let game_id: [u8; 6] = gid_slice.try_into().unwrap();
        unsafe {
            let ids_ptr = ids.as_mut_ptr().cast::<GameID>();
            ids_ptr.add(i).write(GameID::from(game_id));
        }
        cursor += 6;

        // Parse gamehacking id (3 bytes)
        let gh_slice = &input[cursor..cursor + 3];
        let gamehacking_id = u24_le_to_u32(gh_slice.try_into().unwrap());
        ghids[i].write(gamehacking_id);
        cursor += 3;

        // Parse title len
        let str_len = input[cursor] as usize;
        cursor += 1;

        // Parse title string
        let title_slice = &input[cursor..cursor + str_len];
        let title = unsafe { str::from_utf8_unchecked(title_slice) };
        titles[i].write(title);

        cursor += str_len;
        i += 1;
    }

    let ids = unsafe { ids.assume_init() };
    let ghids = unsafe { ghids.assume_init() };
    let titles = unsafe { titles.assume_init() };

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
