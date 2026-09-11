// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fs, path::PathBuf};

fn extract_sha1_list(dat_path: &str) -> Vec<[u8; 20]> {
    let dat = fs::read_to_string(dat_path).unwrap();
    let mut list = Vec::with_capacity(4096);

    let mut remaining = &dat[..];
    while let Some(found) = remaining.find("sha1=\"") {
        remaining = &remaining[found + 6..];

        let next_quote = remaining.find('"').unwrap();
        let sha1 = &remaining[..next_quote];
        assert_eq!(sha1.len(), 40);

        let mut sha1_bytes = [0u8; 20];
        hex::decode_to_slice(sha1, &mut sha1_bytes).unwrap();
        list.push(sha1_bytes);

        remaining = &remaining[next_quote + 1..];
    }

    list
}

fn main() {
    let wii_dat_path = "assets/Nintendo - Wii - Datfile (3780) (2026-06-15 03-13-28).dat";
    let ngc_dat_path = "assets/Nintendo - GameCube - Datfile (2019) (2026-06-13 18-14-01).dat";

    println!("cargo:rerun-if-changed={wii_dat_path}");
    println!("cargo:rerun-if-changed={ngc_dat_path}");
    println!("cargo:rerun-if-changed=build.rs");

    let wii_hashes_handle = std::thread::spawn(|| extract_sha1_list(wii_dat_path));
    let ngc_hashes = extract_sha1_list(ngc_dat_path);
    let wii_hashes = wii_hashes_handle.join().unwrap();

    let mut sha1_list = wii_hashes;
    sha1_list.extend(ngc_hashes);
    sha1_list.sort_unstable();
    sha1_list.dedup();

    let out_path = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("sha1_list.bin");
    fs::write(out_path, sha1_list.as_flattened()).unwrap();
}
