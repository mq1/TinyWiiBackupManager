// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

include!(concat!(env!("OUT_DIR"), "/wiitdb_txt.rs"));

pub fn get(id: [u8; 6]) -> Option<&'static str> {
    WIITDB_TXT
        .binary_search_by_key(&id, |entry| entry.0)
        .map(|i| WIITDB_TXT[i].1)
        .ok()
}
