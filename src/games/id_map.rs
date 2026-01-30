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
    data: Box<[u8]>,
    ids: [GameID; ID_COUNT],
    ghids: Box<[u32]>,
    title_offsets: Box<[usize]>,
}

impl IdMap {
    pub fn get_title(&self, game_id: GameID) -> Option<&str> {
        let i = self.ids.binary_search(&game_id).ok()?;

        unsafe {
            let start = *self.title_offsets.get_unchecked(i);
            let len = *self.data.get_unchecked(start - 1) as usize;
            let slice = self.data.get_unchecked(start..start + len);
            Some(str::from_utf8_unchecked(slice))
        }
    }

    pub fn get_ghid(&self, game_id: GameID) -> Option<u32> {
        let i = self.ids.binary_search(&game_id).ok()?;
        unsafe { Some(*self.ghids.get_unchecked(i)) }
    }
}

pub static ID_MAP: LazyLock<IdMap> = LazyLock::new(deserialize_id_map);

/// lots of unsafe, we assume the input data is correct (it always is)
fn deserialize_id_map() -> IdMap {
    // Decompress
    let mut buf = Box::<[u8]>::new_uninit_slice(ID_MAP_BYTES_LEN);
    let buf_slice: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr().cast(), ID_MAP_BYTES_LEN) };

    let n = zstd::bulk::decompress_to_buffer(COMPRESSED_ID_MAP, buf_slice)
        .expect("Failed to decompress ID map");

    assert!(n == ID_MAP_BYTES_LEN, "Failed to decompress ID map");

    let data = unsafe { buf.assume_init() };

    let mut ids = MaybeUninit::<[GameID; ID_COUNT]>::uninit();
    let mut ghids = Box::<[u32]>::new_uninit_slice(ID_COUNT);
    let mut title_offsets = Box::<[usize]>::new_uninit_slice(ID_COUNT);

    let mut cursor = 0;
    let mut i = 0;

    while i < ID_COUNT {
        // Parse game id (6 bytes)
        let gid_slice = &data[cursor..cursor + 6];
        let game_id: [u8; 6] = gid_slice.try_into().unwrap();
        unsafe {
            let ids_ptr = ids.as_mut_ptr().cast::<GameID>();
            ids_ptr.add(i).write(GameID::from(game_id));
        }
        cursor += 6;

        // Parse gamehacking id (3 bytes)
        let gh_slice = &data[cursor..cursor + 3];
        let gamehacking_id = u24_le_to_u32(gh_slice.try_into().unwrap());
        ghids[i].write(gamehacking_id);
        cursor += 3;

        // Parse title
        let str_len = data[cursor] as usize;
        cursor += 1;
        title_offsets[i].write(cursor);

        cursor += str_len;
        i += 1;
    }

    let ids = unsafe { ids.assume_init() };
    let ghids = unsafe { ghids.assume_init() };
    let title_offsets = unsafe { title_offsets.assume_init() };

    IdMap {
        data,
        ids,
        ghids,
        title_offsets,
    }
}

pub fn get_init_task() -> Task<Message> {
    Task::perform(
        async {
            LazyLock::force(&ID_MAP);
        },
        |()| Message::EmptyResult(Ok(())),
    )
}
