// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

include!(concat!(env!("OUT_DIR"), "/wiitdb_txt.rs"));

pub fn get(id: [u8; 6]) -> Option<&'static str> {
    if id[4] == 0 {
        let id = [id[0], id[1], id[2], id[3]];

        ID4_TITLES
            .binary_search_by_key(&id, |entry| entry.0)
            .map(|i| ID4_TITLES[i].1)
            .ok()
    } else {
        ID6_TITLES
            .binary_search_by_key(&id, |entry| entry.0)
            .map(|i| ID6_TITLES[i].1)
            .ok()
    }
}
