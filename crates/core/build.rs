// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use rkyv::rancor::Error;
use rkyv::{Archive, Serialize};
use std::num::NonZeroU32;
use std::path::Path;
use std::{env, fs};

#[derive(Archive, Serialize)]
struct GameEntry {
    id: u32,
    ghid: Option<NonZeroU32>,
    title: String,
}

fn make_id_map() -> Vec<GameEntry> {
    let contents = fs::read_to_string("../../assets/wiitdb.txt").unwrap();
    let mut lines = contents.lines();

    // skip heading
    let _ = lines.next();

    let mut entries = lines
        .map(|line| {
            let (id, title) = line.split_once(" = ").unwrap();
            let id = u32::from_str_radix(id, 36).unwrap();

            GameEntry {
                id,
                ghid: None,
                title: title.to_string(),
            }
        })
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.id);

    entries
}

fn parse_gamehacking_ids(entries: &mut [GameEntry]) {
    const GHID_ANCHOR: &str = "href=\"/game/";
    const GAMEID_ANCHOR: &str = "<td class=\"text-center\">";

    for i in 0..=70 {
        let filename = format!("../../assets/gamehacking/GameHacking.org - WII - Page {i}.html");
        let content = fs::read_to_string(&filename).unwrap();

        let mut current_slice = &content[..];
        while let Some(ghid_pos) = current_slice.find(GHID_ANCHOR) {
            current_slice = &current_slice[ghid_pos + GHID_ANCHOR.len()..];

            let quote_pos = current_slice.find('"').unwrap();
            let ghid_str = &current_slice[..quote_pos];
            let ghid = NonZeroU32::new(ghid_str.parse().unwrap());

            let gameid_pos = current_slice.find(GAMEID_ANCHOR).unwrap();
            current_slice = &current_slice[gameid_pos + GAMEID_ANCHOR.len()..];
            let td_close_pos = current_slice.find('<').unwrap();
            let gameid_str = current_slice[..td_close_pos].trim();
            if !matches!(gameid_str.len(), 4 | 6) {
                continue;
            }
            let gameid = u32::from_str_radix(gameid_str, 36).unwrap();

            if let Ok(i) = entries.binary_search_by_key(&gameid, |e| e.id) {
                entries[i].ghid = ghid;
            }
        }
    }
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../../assets/wiitdb.txt");
    println!("cargo::rerun-if-changed=../../assets/gamehacking/**");

    let mut entries = make_id_map();
    parse_gamehacking_ids(&mut entries);

    let bytes = rkyv::to_bytes::<Error>(&entries).unwrap();
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("id_map.bin");
    fs::write(out_path, bytes).unwrap();
}
