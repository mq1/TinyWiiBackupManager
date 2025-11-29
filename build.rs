// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn str_to_gameid(id: &str) -> [u8; 6] {
    let mut id_bytes = [0u8; 6];
    let bytes = id.as_bytes();
    let len = bytes.len().min(6);
    id_bytes[..len].copy_from_slice(&bytes[..len]);
    id_bytes
}

fn parse_gamehacking_ids() -> Box<[([u8; 6], u32)]> {
    let file = BufReader::new(File::open("assets/gamehacking-ids.txt").unwrap());
    let mut lines = file.lines();

    let mut ids = Vec::new();
    while let Some(line) = lines.next() {
        let gamehacking_id_raw = line.unwrap();
        let game_id_raw = lines.next().unwrap().unwrap();

        let gamehacking_id = gamehacking_id_raw.trim_start_matches("<td><a href=\"/game/");
        let gamehacking_id_end = gamehacking_id.find('"').unwrap();
        let gamehacking_id = gamehacking_id[..gamehacking_id_end].parse::<u32>().unwrap();

        let game_id = game_id_raw
            .trim_start_matches("<td class=\"text-center\">")
            .trim_end_matches("</td>");

        if !game_id.is_empty() {
            let game_id = str_to_gameid(game_id);
            ids.push((game_id, gamehacking_id));
        }
    }

    let mut ids = ids.into_boxed_slice();
    ids.sort_unstable_by_key(|entry| entry.0);

    ids
}

fn compile_id_map() {
    let gamehacking_ids = parse_gamehacking_ids();
    let file = BufReader::new(File::open("assets/wiitdb.txt").unwrap());
    let mut out_file = BufWriter::new(
        File::create(Path::new(&env::var("OUT_DIR").unwrap()).join("id_map_data.rs")).unwrap(),
    );

    // Skip the first line
    let mut lines = file.lines();
    lines.next();

    // Build the map
    let mut titles_map = Vec::new();

    for line in lines {
        let mut line = line.unwrap();
        let split_pos = line.find('=').unwrap();
        let id = &line[..(split_pos - 1)];
        let id = str_to_gameid(id);
        line.drain(..(split_pos + 2));
        titles_map.push((id, line));
    }

    // Sort the map (to enable binary search)
    titles_map.sort_unstable_by_key(|entry| entry.0);

    // Build the Rust code
    write!(&mut out_file, "const DATA: &[([u8; 6], &str, u32)] = &[").unwrap();
    for (id, title) in titles_map {
        let ghid = gamehacking_ids
            .binary_search_by_key(&id, |entry| entry.0)
            .map(|i| gamehacking_ids[i].1)
            .unwrap_or(0);

        write!(
            &mut out_file,
            "([{},{},{},{},{},{}],\"{}\",{}),",
            id[0], id[1], id[2], id[3], id[4], id[5], title, ghid
        )
        .unwrap();
    }
    write!(&mut out_file, "];").unwrap();
}

fn main() {
    compile_id_map();

    #[cfg(windows)]
    {
        static_vcruntime::metabuild();

        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/TinyWiiBackupManager.ico");
        res.compile().unwrap();
    }
}
