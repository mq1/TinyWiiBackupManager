// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use hex::FromHex;
use std::{env, fs, path::PathBuf};

fn extract_sha1_list(dat_path: &str, list: &mut Vec<[u8; 20]>) {
    const ANCHOR: &str = "sha1=\"";

    let dat = fs::read_to_string(dat_path).unwrap();

    let mut remaining = &dat[..];
    while let Some(found) = remaining.find(ANCHOR) {
        remaining = &remaining[found + ANCHOR.len()..];

        let next_quote = remaining.find('"').unwrap();
        let sha1 = &remaining[..next_quote];
        assert_eq!(sha1.len(), 40);

        let sha1_bytes = <[u8; 20]>::from_hex(sha1).unwrap();
        list.push(sha1_bytes);

        remaining = &remaining[next_quote + 1..];
    }
}

fn main() {
    let wii_dat_path = "assets/Nintendo - Wii - Datfile (3780) (2026-06-15 03-13-28).dat";
    let ngc_dat_path = "assets/Nintendo - GameCube - Datfile (2019) (2026-06-13 18-14-01).dat";

    println!("cargo:rerun-if-changed={wii_dat_path}");
    println!("cargo:rerun-if-changed={ngc_dat_path}");
    println!("cargo:rerun-if-changed=build.rs");

    let mut sha1_list = Vec::with_capacity(8192);
    extract_sha1_list(wii_dat_path, &mut sha1_list);
    extract_sha1_list(ngc_dat_path, &mut sha1_list);

    sha1_list.sort_unstable();
    sha1_list.dedup();

    let mut out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    out_dir.push("sha1_list.bin");

    fs::write(out_dir, sha1_list.as_flattened()).unwrap();
}
