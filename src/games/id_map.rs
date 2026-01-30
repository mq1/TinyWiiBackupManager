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

    // Prepare containers
    let mut ids = MaybeUninit::<[GameID; ID_COUNT]>::uninit();
    let mut ghids = Box::<[u32]>::new_uninit_slice(ID_COUNT);
    let mut title_offsets = Box::<[usize]>::new_uninit_slice(ID_COUNT);

    // Get the raw pointer to the start of the data
    let base_ptr = data.as_ptr();
    let mut offset = 0;
    let mut i = 0;

    // We can hoist the ids pointer calculation out of the loop
    let ids_ptr = ids.as_mut_ptr().cast::<GameID>();

    while i < ID_COUNT {
        unsafe {
            // Calculate pointer to current record
            let curr_ptr = base_ptr.add(offset);

            // Parse Game ID (6 bytes)
            // Cast u8 pointer directly to [u8; 6] pointer and dereference
            let gid_bytes = *curr_ptr.cast::<[u8; 6]>();
            ids_ptr.add(i).write(GameID::from(gid_bytes));

            // Parse GameHacking ID (3 bytes)
            // Offset 6 bytes from current record start
            let gh_bytes = *curr_ptr.add(6).cast::<[u8; 3]>();
            let gh_val = u32::from_le_bytes([gh_bytes[0], gh_bytes[1], gh_bytes[2], 0]);
            ghids[i].write(gh_val);

            // Parse Title Length (1 byte)
            // Offset 9 bytes (6 + 3)
            let str_len = *curr_ptr.add(9) as usize;

            // Store the Title Offset
            // The string bytes start 10 bytes in (6 + 3 + 1)
            title_offsets[i].write(offset + 10);

            // Advance offset: 10 bytes of headers + length of string
            offset += 10 + str_len;
            i += 1;
        }
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
