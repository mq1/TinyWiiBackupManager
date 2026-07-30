// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

fn extract_sha1_list(dat_path: &str, set: &mut BTreeSet<[u8; 20]>) {
    let dat = fs::read_to_string(dat_path).unwrap();

    let mut remaining = &dat[..];
    while let Some(found) = remaining.find("sha1=\"") {
        remaining = &remaining[found + 6..];
        let next_quote = remaining.find('"').unwrap();
        let sha1 = &remaining[..next_quote];

        assert_eq!(sha1.len(), 40);

        let mut sha1_bytes = [0u8; 20];
        hex::decode_to_slice(sha1, &mut sha1_bytes).unwrap();
        set.insert(sha1_bytes);

        remaining = &remaining[next_quote + 1..];
    }
}

fn main() {
    let mut sha1_list = BTreeSet::new();

    let wii_dat_path = "assets/Nintendo - Wii - Datfile (3780) (2026-06-15 03-13-28).dat";
    extract_sha1_list(wii_dat_path, &mut sha1_list);

    let ngc_dat_path = "assets/Nintendo - GameCube - Datfile (2019) (2026-06-13 18-14-01).dat";
    extract_sha1_list(ngc_dat_path, &mut sha1_list);

    let mut out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    out_path.push("sha1_list.bin");

    let out = File::create(out_path).unwrap();
    let mut out = BufWriter::new(out);
    for sha1 in &sha1_list {
        out.write_all(sha1).unwrap();
    }
}
