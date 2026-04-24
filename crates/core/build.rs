// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::Path;
use std::{env, fmt::Write, fs};

fn parse_titles_txt() -> Vec<(u32, String)> {
    let mut title_map = Vec::new();

    let contents = fs::read_to_string("../../assets/wiitdb.txt").unwrap();
    let mut lines = contents.lines();

    // skip heading
    let _ = lines.next();

    for line in lines {
        let (gameid, title) = line.split_once(" = ").unwrap();
        let gameid = u32::from_str_radix(gameid, 36).unwrap();
        title_map.push((gameid, title.to_string()));
    }

    title_map.sort_by_key(|(game_id, _)| *game_id);

    title_map
}

fn parse_gamehacking_ids() -> Vec<(u32, u32)> {
    const GHID_ANCHOR: &str = "href=\"/game/";
    const GAMEID_ANCHOR: &str = "<td class=\"text-center\">";

    let mut id_map = Vec::new();

    for i in 0..=70 {
        let filename = format!("../../assets/gamehacking/GameHacking.org - WII - Page {i}.html");
        let content = fs::read_to_string(&filename).unwrap();

        let mut current_slice = &content[..];
        while let Some(ghid_pos) = current_slice.find(GHID_ANCHOR) {
            current_slice = &current_slice[ghid_pos + GHID_ANCHOR.len()..];

            let quote_pos = current_slice.find('"').unwrap();
            let ghid_str = &current_slice[..quote_pos];
            let ghid = ghid_str.parse().unwrap();
            if ghid == 0 {
                continue;
            }

            let gameid_pos = current_slice.find(GAMEID_ANCHOR).unwrap();
            current_slice = &current_slice[gameid_pos + GAMEID_ANCHOR.len()..];
            let td_close_pos = current_slice.find('<').unwrap();
            let gameid_str = current_slice[..td_close_pos].trim();
            if !matches!(gameid_str.len(), 4 | 6) {
                continue;
            }
            let gameid = u32::from_str_radix(gameid_str, 36).unwrap();

            id_map.push((gameid, ghid));
        }
    }

    id_map
}

fn make_id_map() {
    let title_map = parse_titles_txt();
    let gamehacking_ids = parse_gamehacking_ids();

    let mut entries = Vec::new();
    for (id, title) in title_map {
        let ghid = gamehacking_ids
            .iter()
            .find(|(gameid, _)| *gameid == id)
            .map(|(_, ghid)| *ghid)
            .unwrap_or(0);

        entries.push((id, ghid, title));
    }

    let mut code =
        String::from("#[allow(clippy::unreadable_literal)]\nconst GAMES:&[GameEntry]=&[");
    for (id, ghid, title) in entries {
        write!(code, "GameEntry::new({id},{ghid},{title:?}),").unwrap();
    }
    code.push_str("];");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("id_map_generated.rs");
    fs::write(out_path, code).unwrap();
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=../../assets/wiitdb.txt");
    println!("cargo::rerun-if-changed=../../assets/gamehacking/**");

    make_id_map();
}
