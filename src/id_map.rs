// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Clone, Copy)]
pub struct GameEntry {
    id: [u8; 6],
    pub ghid: Option<u32>,
    pub title: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/id_map_generated.rs"));

pub fn get(id: &str) -> Option<&'static GameEntry> {
    let mut buf = [0; 6];
    let bytes = id.as_bytes();
    let len = bytes.len().min(6);
    buf[..len].copy_from_slice(&bytes[..len]);

    let i = GAMES.binary_search_by_key(&buf, |e| e.id).ok()?;
    let entry = &GAMES[i];

    Some(entry)
}
