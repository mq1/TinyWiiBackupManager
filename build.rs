// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

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

fn u32_to_u24_le(n: u32) -> [u8; 3] {
    let bytes = n.to_le_bytes();
    [bytes[0], bytes[1], bytes[2]]
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
        let contents = fs::read_to_string(filename).unwrap();

        for cap in re.captures_iter(&contents) {
            if let Some(game_id) = str_to_game_id(&cap[2])
                && let Ok(ghid) = cap[1].parse()
            {
                id_map.push((game_id, ghid));
            }
        }
    }

    id_map.sort_by_key(|(game_id, _)| *game_id);

    id_map
}

fn serialize_id_map() {
    let title_map = parse_titles_txt();
    let gamehacking_ids = parse_gamehacking_ids();

    let mut out = Vec::new();

    let id_count = title_map.len();
    for (game_id, title) in title_map {
        let ghid = gamehacking_ids
            .iter()
            .find(|(id, _)| *id == game_id)
            .map(|(_, ghid)| ghid)
            .copied()
            .unwrap_or(0);

        let title_len: u8 = title.len().try_into().unwrap();

        out.write_all(&game_id).unwrap();
        out.write_all(&u32_to_u24_le(ghid)).unwrap();
        out.write_all(&[title_len]).unwrap();
        out.write_all(title.as_bytes()).unwrap();
    }

    let compressed = zstd::bulk::compress(&out, 19).unwrap();
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("id_map.bin.zst");
    fs::write(out_path, compressed).unwrap();

    let meta = format!(
        "const ID_COUNT: usize = {};\n#[allow(clippy::unreadable_literal)]\nconst ID_MAP_BYTES_LEN: usize = {};",
        id_count,
        out.len()
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
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/wiitdb.txt");
    println!("cargo::rerun-if-changed=assets/gamehacking-ids.txt");

    serialize_id_map();
    compress_lucide();

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.13");

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=11.0");

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("package/windows/TinyWiiBackupManager.ico");
        res.compile().unwrap();
    }
}
