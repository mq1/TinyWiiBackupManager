// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::game_id::GameID;
use crate::message::Message;
use iced::Task;
use std::str;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

const COMPRESSED_TITLE_MAP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/title_map.bin.zst"));

static TITLE_MAP: LazyLock<Box<[u8]>> = LazyLock::new(|| {
    let mut buf = Box::<[u8]>::new_uninit_slice(TITLES_LEN);

    let buf_slice: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr().cast(), TITLES_LEN) };

    let _ = zstd::bulk::decompress_to_buffer(COMPRESSED_TITLE_MAP, buf_slice)
        .expect("Failed to decompress title map");

    unsafe { buf.assume_init() }
});

pub fn get_title(game_id: GameID) -> Option<&'static str> {
    let inner = game_id.inner();

    let i = GAME_IDS.binary_search_by_key(&inner, |id| *id).ok()?;
    let offset = unsafe { *TITLE_OFFSETS.get_unchecked(i) } as usize;
    let next_offset = unsafe { *TITLE_OFFSETS.get_unchecked(i + 1) } as usize;

    let title = unsafe { str::from_utf8_unchecked(&TITLE_MAP[offset..next_offset]) };
    Some(title)
}

pub fn get_ghid(game_id: GameID) -> Option<u32> {
    let inner = game_id.inner();

    let i = GAME_IDS.binary_search_by_key(&inner, |id| *id).ok()?;
    let ghid = unsafe { *GAMEHACKING_IDS.get_unchecked(i) };

    Some(ghid)
}

pub fn get_init_task() -> Task<Message> {
    Task::perform(
        async {
            LazyLock::force(&TITLE_MAP);
        },
        |()| Message::EmptyResult(Ok(())),
    )
}
