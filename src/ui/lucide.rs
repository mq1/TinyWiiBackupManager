// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::{Task, font};

include!(concat!(env!("OUT_DIR"), "/lucide_meta.rs"));
const COMPRESSED_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/lucide.ttf.zst"));

pub fn get_load_lucide_task() -> Task<Message> {
    Task::future(async { load_lucide() })
        .then(font::load)
        .map_err(|e| format!("{e:#?}"))
        .map(Message::EmptyResult)
}

fn load_lucide() -> Vec<u8> {
    zstd::bulk::decompress(COMPRESSED_BYTES, LUCIDE_BYTES_LEN)
        .expect("Failed to decompress lucide.ttf.zst")
}
