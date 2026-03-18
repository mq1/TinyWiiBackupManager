// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::{Task, font};

include!(concat!(env!("OUT_DIR"), "/lucide_meta.rs"));

pub fn get_load_lucide_task() -> Task<Message> {
    Task::future(get_font_bytes())
        .then(font::load)
        .map_err(|e| format!("Failed to load lucide.ttf: {e:#?}"))
        .map(Message::EmptyResult)
}

#[allow(clippy::unused_async)]
async fn get_font_bytes() -> Vec<u8> {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/lucide.ttf"));

    #[cfg(feature = "compress-idmap")]
    {
        zstd::bulk::decompress(bytes, UNCOMPRESSED_SIZE).unwrap()
    }

    #[cfg(not(feature = "compress-idmap"))]
    {
        bytes.to_vec()
    }
}
