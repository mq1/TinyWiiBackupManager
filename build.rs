// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::num::NonZeroU32;
use std::path::Path;
use std::{collections::HashMap, env, fmt::Write, fs, path::PathBuf};

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

fn parse_gamehacking_ids() -> Vec<([u8; 6], NonZeroU32)> {
    let mut id_map = Vec::new();

    for i in 0..=70 {
        let filename = format!("assets/gamehacking/GameHacking.org - WII - Page {i}.html");
        let contents = fs::read_to_string(filename).unwrap();

        let matches = contents.match_indices("<a href=\"game/");
        for (i, _) in matches {
            let window = &contents[i..];

            let ghid_start = 15;
            let ghid_end = window.find("\">").unwrap();

            let Ok(ghid) = window[ghid_start..ghid_end].parse() else {
                continue;
            };

            let (gameid_start, _) = window.match_indices("\">").nth(2).unwrap();
            let (gameid_end, _) = window.match_indices("</td>").nth(2).unwrap();
            let gameid = &window[gameid_start + 1..gameid_end];

            if matches!(gameid.len(), 4 | 6) {
                id_map.push((parse_gameid(gameid), NonZeroU32::new(ghid).unwrap()));
            }
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
            .map(|(_, ghid)| *ghid);
        entries.push((id, ghid, title));
    }

    let mut code = String::from("const GAMES:&[GameEntry]=unsafe{&[");
    for (id, ghid, title) in entries {
        match ghid {
            Some(ghid) => {
                write!(
                    code,
                    "GameEntry{{id:{id:?},ghid:Some(NonZeroU32::new_unchecked({ghid})),title:{title:?}}},"
                )
                .unwrap();
            }
            None => {
                write!(code, "GameEntry{{id:{id:?},ghid:None,title:{title:?}}},").unwrap();
            }
        }
    }
    code.push_str("]};");

    let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join("id_map_generated.rs");
    fs::write(out_path, code).unwrap();
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/wiitdb.txt");
    println!("cargo::rerun-if-changed=assets/gamehacking/**");
    println!("cargo::rerun-if-changed=ui/**");

    make_id_map();

    let library = HashMap::from([("lucide".to_string(), PathBuf::from(lucide_slint::lib()))]);
    let config = slint_build::CompilerConfiguration::new().with_library_paths(library);

    slint_build::compile_with_config("ui/app-window.slint", config).unwrap();

    let target = env::var("TARGET").unwrap();

    if target.contains("-windows-") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("package/windows/icon.ico");
        res.set_manifest_file("package/windows/TinyWiiBackupManager.exe.manifest");
        res.compile().unwrap();
    }
}
