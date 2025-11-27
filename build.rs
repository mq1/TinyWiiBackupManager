// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;
use std::{env, fs};

fn id_str_to_bytes(id: &str) -> [u8; 6] {
    let mut id_bytes = [0u8; 6];
    let bytes = id.as_bytes();
    let len = bytes.len().min(6);
    id_bytes[..len].copy_from_slice(&bytes[..len]);
    id_bytes
}

fn compile_wiitdb_txt() {
    let txt = fs::read_to_string("assets/wiitdb.txt").unwrap();

    // Skip the first line
    let mut lines = txt.lines();
    lines.next();

    // Build the map
    let mut titles = Vec::new();
    for line in lines {
        let (id, title) = line.split_once(" = ").unwrap();
        titles.push((id_str_to_bytes(id), title));
    }

    // Sort the map (to enable binary search)
    titles.sort_unstable_by_key(|entry| entry.0);

    // Build the Rust code
    let mut wiitdb_txt_rs = "const WIITDB_TXT: &[([u8; 6], &str)] = &[".to_string();

    for (id, title) in titles {
        wiitdb_txt_rs.push_str(&format!(
            "([{},{},{},{},{},{}],\"{}\"),",
            id[0], id[1], id[2], id[3], id[4], id[5], title
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
