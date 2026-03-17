// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::{Task, font};

include!(concat!(env!("OUT_DIR"), "/lucide_meta.rs"));
const COMPRESSED_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/lucide.ttf.zst"));

pub fn get_load_lucide_task() -> Task<Message> {
    Task::future(async { zstd::bulk::decompress(COMPRESSED_BYTES, LUCIDE_BYTES_LEN).unwrap() })
        .then(font::load)
        .map_err(|e| format!("Failed to load lucide.ttf: {e:#?}"))
        .map(Message::EmptyResult)
}
