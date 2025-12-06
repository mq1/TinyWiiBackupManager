// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

include!(concat!(env!("OUT_DIR"), "/id_map_data.rs"));

pub fn get_title(id: [u8; 6]) -> Option<&'static str> {
    DATA.binary_search_by_key(&id, |entry| entry.0)
        .map(|i| DATA[i].1)
        .ok()
}

pub fn get_gamehacking_id(id: [u8; 6]) -> Option<u32> {
    DATA.binary_search_by_key(&id, |entry| entry.0)
        .map(|i| DATA[i].2)
        .ok()
}
