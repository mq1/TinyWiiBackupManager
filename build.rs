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
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("gamehacking_ids.rs");
    let mut out_file = BufWriter::new(File::create(out_path).unwrap());

    // Build the Rust code
    write!(
        &mut out_file,
        "const GAMEID_TO_GHID: &[([u8; 6], u32)] = &["
    )
    .unwrap();
    for (game_id, ghid) in gamehacking_ids {
        write!(
            &mut out_file,
            "([{},{},{},{},{},{}],{}),",
            game_id[0], game_id[1], game_id[2], game_id[3], game_id[4], game_id[5], ghid
        )
        .unwrap();
    }
    write!(&mut out_file, "];").unwrap();
}

fn main() {
    compile_id_map();

    #[cfg(target_vendor = "pc")]
    static_vcruntime::metabuild();

    #[cfg(windows)]
    let mut res = winresource::WindowsResource::new();

    #[cfg(windows)]
    res.set_icon("assets/TinyWiiBackupManager.ico");

    #[cfg(target_vendor = "pc")]
    res.set_manifest_file("assets/TinyWiiBackupManager.exe.manifest");

    #[cfg(windows)]
    res.compile().unwrap();
}
