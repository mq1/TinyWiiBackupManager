// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::cast_possible_truncation)]

use lucide_icons::LUCIDE_FONT_BYTES;
use regex::Regex;
use std::io::Write;
use std::path::Path;
use std::{env, fs};

fn str_to_game_id(id: &str) -> Option<[u8; 6]> {
    let id = id.as_bytes();

    match id.len() {
        6 => Some([id[0], id[1], id[2], id[3], id[4], id[5]]),
        4 => Some([id[0], id[1], id[2], id[3], 0, 0]),
        _ => None,
    }
}

fn parse_titles_txt() -> Vec<([u8; 6], String)> {
    let mut title_map = Vec::new();

    let contents = fs::read_to_string("assets/wiitdb.txt").unwrap();
    let mut lines = contents.lines();

    // skip heading
    let _ = lines.next();

    for line in lines {
        let (game_id, title) = line.split_once(" = ").unwrap();
        let game_id = str_to_game_id(game_id).unwrap();
        title_map.push((game_id, title.to_string()));
    }

    title_map.sort_by_key(|(game_id, _)| *game_id);

    title_map
}

fn parse_gamehacking_ids() -> Vec<([u8; 6], u32)> {
    let mut id_map = Vec::new();

    let re =
        Regex::new(r#"(?s)href="/game/(\d+)"[^>]*>.*?<td[^>]*>\s*([A-Z0-9]+)\s*</td>"#).unwrap();

    for i in 0..=70 {
        let filename = format!("assets/gamehacking/GameHacking.org - WII - Page {i}.html");
        println!("cargo:rerun-if-changed={filename}");
        let contents = fs::read_to_string(filename).unwrap();

        for cap in re.captures_iter(&contents) {
            if let Some(game_id) = str_to_game_id(&cap[2])
                && let Ok(ghid) = cap[1].parse()
            {
                id_map.push((game_id, ghid));
            }
        }
    }

    id_map
}

fn make_id_map() {
    let title_map = parse_titles_txt();
    let gamehacking_ids = parse_gamehacking_ids();

    let mut filled_ghids = Vec::new();
    for (game_id, _) in &title_map {
        let ghid = gamehacking_ids
            .iter()
            .find(|(id, _)| *id == *game_id)
            .map(|(_, ghid)| ghid)
            .copied()
            .unwrap_or(0);

        filled_ghids.push(ghid);
    }

    let mut data = Vec::new();

    // Write ordered game ids
    for (game_id, _) in &title_map {
        data.write_all(game_id).unwrap();
    }

    // Write ordered gamehacking ids
    for id in &filled_ghids {
        let id = id.to_le_bytes();
        data.write_all(&[id[0], id[1], id[2]]).unwrap();
    }

    // Write ordered title lengths
    for (_, title) in &title_map {
        let len = title.len() as u8;
        data.write_all(&[len]).unwrap();
    }

    // Write ordered titles
    for (_, title) in &title_map {
        data.write_all(title.as_bytes()).unwrap();
    }

    let compressed_data = zstd::bulk::compress(&data, 19).unwrap();
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("id_map.bin.zst");
    fs::write(out_path, compressed_data).unwrap();

    let meta = format!(
        "#[allow(clippy::unreadable_literal)]\nconst ID_MAP_LEN: usize = {};\n#[allow(clippy::unreadable_literal)]\nconst DATA_SIZE: usize = {};",
        title_map.len(),
        data.len()
    );

    let meta_out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("id_map_meta.rs");
    fs::write(meta_out_path, meta).unwrap();
}

fn compress_lucide() {
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("lucide.ttf.zst");
    let meta_out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("lucide_meta.rs");

    let bytes = zstd::bulk::compress(LUCIDE_FONT_BYTES, 19).unwrap();
    fs::write(out_path, bytes).unwrap();

    let meta = format!(
        "#[allow(clippy::unreadable_literal)]\nconst LUCIDE_BYTES_LEN: usize = {};",
        LUCIDE_FONT_BYTES.len()
    );
    fs::write(meta_out_path, meta).unwrap();
}

fn main() {
    println!("cargo:rerun-if-changed=assets/wiitdb.txt");
    println!("cargo:rerun-if-changed=assets/gamehacking-ids.txt");

    make_id_map();
    compress_lucide();

    if env::var("TARGET").unwrap() == "x86_64-apple-darwin" {
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");
    }

    if env::var("TARGET").unwrap() == "aarch64-apple-darwin" {
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=11.0");
    }

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("package/windows/icon.ico");
        res.compile().unwrap();
    }
}
