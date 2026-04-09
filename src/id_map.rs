// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use rkyv::vec::ArchivedVec;

const DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/id_map.bin"));

#[derive(rkyv::Archive, rkyv::Deserialize)]
pub struct GameEntry {
    id: [u8; 6],
    pub ghid: Option<u32>,
    pub title: String,
}

pub fn get(id: &str) -> Option<&ArchivedGameEntry> {
    let id = id.as_bytes();
    let mut buf = [0; 6];
    buf[..id.len()].copy_from_slice(id);

    let map = unsafe { rkyv::access_unchecked::<ArchivedVec<ArchivedGameEntry>>(DATA) };
    let i = map.binary_search_by_key(&buf, |entry| entry.id).ok()?;
    let entry = &map[i];

    Some(entry)
}
