// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::{env, fs};

fn str_to_gameid(id: &str) -> [u8; 6] {
    let mut id_bytes = [0u8; 6];
    let bytes = id.as_bytes();
    let len = bytes.len().min(6);
    id_bytes[..len].copy_from_slice(&bytes[..len]);
    id_bytes
}

fn parse_gamehacking_ids() -> Box<[([u8; 6], u32)]> {
    let txt = fs::read_to_string("assets/gamehacking-ids.txt").unwrap();

    let mut ids = Vec::new();
    let lines = txt.lines().count();
    for i in (0..lines).step_by(2) {
        let gamehacking_id_raw = txt.lines().nth(i).unwrap();
        let game_id_raw = txt.lines().nth(i + 1).unwrap();

        let gamehacking_id = gamehacking_id_raw
            .trim()
            .trim_start_matches("<td><a href=\"/game/");
        let gamehacking_id_end = gamehacking_id.find('"').unwrap();
        let gamehacking_id = gamehacking_id[..gamehacking_id_end].to_string();
        let gamehacking_id = u32::from_str_radix(&gamehacking_id, 10).unwrap();

        let game_id = game_id_raw
            .trim()
            .trim_start_matches("<td class=\"text-center\">")
            .trim_end_matches("</td>");

        let game_id = str_to_gameid(&game_id);
        ids.push((game_id, gamehacking_id));
    }

    let mut ids = ids.into_boxed_slice();
    ids.sort_unstable_by_key(|entry| entry.0);

    ids
}

fn compile_id_map() {
    let gamehacking_ids = parse_gamehacking_ids();

    let txt = fs::read_to_string("assets/wiitdb.txt").unwrap();

    // Skip the first line
    let mut lines = txt.lines();
    lines.next();

    // Build the map
    let mut id6_titles = Vec::new();

    for line in lines {
        let (id, title) = line.split_once(" = ").unwrap();
        id6_titles.push((str_to_gameid(id), title));
    }

    // Sort the map (to enable binary search)
    id6_titles.sort_unstable_by_key(|entry| entry.0);

    // Build the Rust code
    let mut id_map_data = String::new();

    id_map_data.push_str("const DATA: &[([u8; 6], &str, u32)] = &[");
    for (id, title) in id6_titles {
        let ghid = gamehacking_ids
            .binary_search_by_key(&id, |entry| entry.0)
            .map(|i| gamehacking_ids[i].1)
            .unwrap_or(0);

        id_map_data.push_str(&format!(
            "([{},{},{},{},{},{}],\"{}\",{}),",
            id[0], id[1], id[2], id[3], id[4], id[5], title, ghid
        ));
    }
    id_map_data.push_str("];");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("id_map_data.rs"), id_map_data).unwrap();
}

fn main() {
    compile_id_map();
    println!("cargo:rerun-if-changed=assets/wiitdb.txt");

    // Windows-specific icon resource
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/TinyWiiBackupManager.ico");
        res.compile().unwrap();
    }
}
