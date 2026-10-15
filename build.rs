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

    if cfg!(feature = "dist-windows-modern") || cfg!(feature = "dist-windows-legacy") {
        let project_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();

        if !std::fs::exists(format!("{project_dir}\\VC-LTL-Binary")).unwrap() {
            let _ = std::process::Command::new("curl.exe")
                .args(["-sSLo", "VC-LTL-Binary.7z", "https://github.com/Chuyu-Team/VC-LTL5/releases/download/v5.3.1/VC-LTL-Binary.7z"])
                .status()
                .unwrap();

            let _ = std::process::Command::new("7z.exe")
                .args(["x", "VC-LTL-Binary.7z", "-oVC-LTL-Binary"])
                .status()
                .unwrap();
        }

        let vc_ltl5_path = match target_arch.as_str() {
            "x86_64" => "VC-LTL-Binary\\TargetPlatform\\6.0.6000.0\\lib\\x64",
            "x86" => "VC-LTL-Binary\\TargetPlatform\\5.1.2600.0\\lib\\Win32",
            "aarch64" => "VC-LTL-Binary\\TargetPlatform\\10.0.10240.0\\lib\\ARM64",
            _ => unreachable!(),
        };
        println!("cargo:rustc-link-search=native={project_dir}\\{vc_ltl5_path}");

        if target_arch != "aarch64" {
            if !std::fs::exists(format!("{project_dir}\\YY-Thunks-Objs")).unwrap() {
                let _ = std::process::Command::new("curl.exe")
                    .args(["-sSLo", "YY-Thunks-Objs.zip", "https://github.com/Chuyu-Team/YY-Thunks/releases/download/v1.2.3/YY-Thunks-Objs.zip"])
                    .status()
                    .unwrap();

                let _ = std::process::Command::new("7z.exe")
                    .args(["x", "YY-Thunks-Objs.zip", "-oYY-Thunks-Objs"])
                    .status()
                    .unwrap();
            }

            let yy_thunks_path = match target_arch.as_str() {
                "x86_64" => "YY-Thunks-Objs\\objs\\x64\\YY_Thunks_for_Vista.obj",
                "x86" => "YY-Thunks-Objs\\objs\\x86\\YY_Thunks_for_WinXP.obj",
                _ => unreachable!(),
            };
            println!("cargo:rustc-link-arg={project_dir}\\{yy_thunks_path}");
            println!("cargo:rustc-link-lib=dylib=ole32");

            match target_arch.as_str() {
                "x86_64" => println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS,6.00"),
                "x86" => println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS,5.01"),
                _ => unreachable!(),
            }
        }
    }
}
