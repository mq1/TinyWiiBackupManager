// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use lucide_icons::LUCIDE_FONT_BYTES;
use regex::Regex;
use std::fs::File;
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

fn parse_gamehacking_ids() -> Vec<([u8; 6], u32)> {
    let mut id_map = Vec::new();

    let re =
        Regex::new(r#"(?s)href="/game/(\d+)"[^>]*>.*?<td[^>]*>\s*([A-Z0-9]+)\s*</td>"#).unwrap();

    for i in 0..=70 {
        let filename = format!("assets/gamehacking/GameHacking.org | WII | Page {i}.html");
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

fn compile_id_map() {
    let gamehacking_ids = parse_gamehacking_ids();
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("gamehacking_ids.rs");
    let mut out_file = File::create(out_path).unwrap();

    write!(
        &mut out_file,
        "#[allow(clippy::unreadable_literal)]\nconst GAMEID_TO_GHID: &[([u8; 6], u32)] = &["
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
    println!("cargo::rerun-if-changed=assets/gamehacking-ids.txt");

    compile_id_map();
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
