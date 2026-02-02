// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

// There is no reason for this code to be this low level
// I just wanted to have some fun optimizing stuff

use crate::games::game_id::GameID;
use crate::message::Message;
use iced::Task;
use std::str;
use std::sync::LazyLock;

include!(concat!(env!("OUT_DIR"), "/id_map_meta.rs"));

const COMPRESSED_ID_MAP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin.zst"));
const GAMEHACKING_IDS_OFFSET: usize = ID_MAP_LEN * 6;
const TITLE_LENGTHS_OFFSET: usize = GAMEHACKING_IDS_OFFSET + ID_MAP_LEN * 3;
const TITLES_OFFSET: usize = TITLE_LENGTHS_OFFSET + ID_MAP_LEN;

static ID_MAP: LazyLock<Box<[u8]>> = LazyLock::new(|| {
    zstd::bulk::decompress(COMPRESSED_ID_MAP, DATA_SIZE)
        .expect("Failed to decompress id map")
        .into_boxed_slice()
});

#[inline]
fn gameid_slice() -> &'static [[u8; 6]] {
    let ptr = ID_MAP.as_ptr().cast::<[u8; 6]>();
    unsafe { std::slice::from_raw_parts(ptr, ID_MAP_LEN) }
}

#[inline]
fn get_ghid_at(i: usize) -> u32 {
    let ptr = unsafe { ID_MAP.as_ptr().add(GAMEHACKING_IDS_OFFSET + i * 3) };
    let raw = unsafe { ptr.cast::<u32>().read_unaligned() };
    u32::from_le(raw) & 0x00FF_FFFF
}

static TITLES: LazyLock<Box<[&'static str]>> = LazyLock::new(|| {
    let mut titles = Box::new_uninit_slice(ID_MAP_LEN);

    let mut data_ptr = unsafe { ID_MAP.as_ptr().add(TITLE_LENGTHS_OFFSET) };

    let mut cursor = TITLES_OFFSET;
    for title_ref in &mut titles {
        let title_end = cursor + unsafe { *data_ptr } as usize;
        let title = unsafe { str::from_utf8_unchecked(&ID_MAP[cursor..title_end]) };
        title_ref.write(title);

        data_ptr = unsafe { data_ptr.add(1) };
        cursor = title_end;
    }

    unsafe { titles.assume_init() }
});

pub fn get_title(game_id: GameID) -> Option<&'static str> {
    let inner = game_id.inner();
    let i = gameid_slice().binary_search_by_key(&inner, |id| *id).ok()?;
    Some(unsafe { *TITLES.get_unchecked(i) })
}

pub fn get_ghid(game_id: GameID) -> Option<u32> {
    let inner = game_id.inner();
    let i = gameid_slice().binary_search_by_key(&inner, |id| *id).ok()?;

    match get_ghid_at(i) {
        0 => None,
        ghid => Some(ghid),
    }
}

pub fn get_init_task() -> Task<Message> {
    Task::perform(
        async {
            LazyLock::force(&ID_MAP);
            LazyLock::force(&TITLES);
        },
        |()| Message::EmptyResult(Ok(())),
    )
}
