// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use regex::Regex;
use std::path::Path;
use std::{collections::HashMap, env, fs, path::PathBuf};

#[derive(rkyv::Archive, rkyv::Serialize)]
struct GameEntry {
    id: [u8; 6],
    ghid: Option<u32>,
    title: String,
}

fn parse_gameid(id: &str) -> [u8; 6] {
    let bytes = id.as_bytes();
    let mut buf = [0; 6];
    buf[..bytes.len()].copy_from_slice(bytes);
    buf
}

fn parse_titles_txt() -> Vec<([u8; 6], String)> {
    let mut title_map = Vec::new();

    let contents = fs::read_to_string("assets/wiitdb.txt").unwrap();
    let mut lines = contents.lines();

    // skip heading
    let _ = lines.next();

    for line in lines {
        let (gameid, title) = line.split_once(" = ").unwrap();
        let gameid = parse_gameid(gameid);
        title_map.push((gameid, title.to_string()));
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
            if cap[1].is_empty() || cap[2].is_empty() {
                continue;
            }

            let gameid = parse_gameid(&cap[2]);
            let ghid = cap[1].parse().unwrap();
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
            .find(|(game_id, _)| *game_id == id)
            .map(|(_, ghid)| *ghid);

        entries.push(GameEntry { id, ghid, title });
    }

    let data = rkyv::to_bytes::<rkyv::rancor::Error>(&entries).unwrap();
    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("id_map.bin");
    fs::write(out_path, &data).unwrap();
}

fn main() {
    make_id_map();

    let library = HashMap::from([("lucide".to_string(), PathBuf::from(lucide_slint::lib()))]);
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(library)
        .with_style("cupertino".into());

    slint_build::compile_with_config("ui/app-window.slint", config).unwrap();

    let target = env::var("TARGET").unwrap();

    if target.contains("-windows-") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("package/windows/icon.ico");
        res.set_manifest_file("package/windows/TinyWiiBackupManager.exe.manifest");
        res.compile().unwrap();
    }
}
