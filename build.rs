// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::{env, fs};

fn id6_str_to_bytes(id: &str) -> [u8; 6] {
    let mut id_bytes = [0u8; 6];
    id_bytes.copy_from_slice(id.as_bytes());
    id_bytes
}

fn id4_str_to_bytes(id: &str) -> [u8; 4] {
    let mut id_bytes = [0u8; 4];
    id_bytes.copy_from_slice(id.as_bytes());
    id_bytes
}

fn compile_wiitdb_txt() {
    let txt = fs::read_to_string("assets/wiitdb.txt").unwrap();

    // Skip the first line
    let mut lines = txt.lines();
    lines.next();

    // Build the maps
    let mut id6_titles = Vec::new();
    let mut id4_titles = Vec::new();

    for line in lines {
        let (id, title) = line.split_once(" = ").unwrap();

        match id.len() {
            6 => id6_titles.push((id6_str_to_bytes(id), title)),
            4 => id4_titles.push((id4_str_to_bytes(id), title)),
            _ => panic!("Invalid ID: {}", id),
        }
    }

    // Sort the maps (to enable binary search)
    id6_titles.sort_unstable_by_key(|entry| entry.0);
    id4_titles.sort_unstable_by_key(|entry| entry.0);

    // Build the Rust code
    let mut wiitdb_txt_rs = String::new();

    wiitdb_txt_rs.push_str("const ID6_TITLES: &[([u8; 6], &str)] = &[");
    for (id, title) in id6_titles {
        wiitdb_txt_rs.push_str(&format!(
            "([{},{},{},{},{},{}],\"{}\"),",
            id[0], id[1], id[2], id[3], id[4], id[5], title
        ));
    }
    wiitdb_txt_rs.push_str("];");

    wiitdb_txt_rs.push_str("const ID4_TITLES: &[([u8; 4], &str)] = &[");
    for (id, title) in id4_titles {
        wiitdb_txt_rs.push_str(&format!(
            "([{},{},{},{}],\"{}\"),",
            id[0], id[1], id[2], id[3], title
        ));
    }
    wiitdb_txt_rs.push_str("];");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("wiitdb_txt.rs"), wiitdb_txt_rs).unwrap();
}

fn main() {
    compile_wiitdb_txt();
    println!("cargo:rerun-if-changed=assets/wiitdb.txt");

    // Windows-specific icon resource
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/TinyWiiBackupManager.ico");
        res.compile().unwrap();
    }
}
