// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, fs, path::PathBuf};
use winresource::WindowsResource;
use zip::ZipArchive;

fn extract_sha1_list(dat_path: &str, list: &mut Vec<[u8; 20]>) {
    const ANCHOR: &str = "sha1=\"";

    let dat = fs::read_to_string(dat_path).unwrap();

    let mut remaining = &dat[..];
    while let Some(found) = remaining.find(ANCHOR) {
        remaining = &remaining[found + ANCHOR.len()..];

        let next_quote = remaining.find('"').unwrap();
        let sha1 = &remaining[..next_quote];
        assert_eq!(sha1.len(), 40);

        let mut sha1_bytes = [0u8; 20];
        hex::decode_to_slice(sha1, &mut sha1_bytes).unwrap();
        list.push(sha1_bytes);

        remaining = &remaining[next_quote + 1..];
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

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

    fs::write(out_dir.join("sha1_list.bin"), sha1_list.as_flattened()).unwrap();

    // lighter and static vcruntime replacement
    if cfg!(feature = "static-vcrt") {
        let vc_ltl5_dir = out_dir.join("VC-LTL-Binary");

        if !vc_ltl5_dir.exists() {
            let resp = minreq::get(
                "https://github.com/Chuyu-Team/VC-LTL5/releases/download/v5.3.1/VC-LTL-Binary.7z",
            )
            .with_timeout(60)
            .send()
            .unwrap();
            assert_eq!(resp.status_code, 200);

            fs::create_dir_all(&vc_ltl5_dir).unwrap();
            let mut cursor = std::io::Cursor::new(resp.as_bytes());
            sevenz_rust2::decompress(&mut cursor, &vc_ltl5_dir).unwrap();
        }

        let (win_ver, arch) = match target_arch.as_str() {
            "x86_64" => ("6.0.6000.0", "x64"),      // vista
            "x86" => ("5.1.2600.0", "Win32"),       // xp
            "aarch64" => ("10.0.10240.0", "ARM64"), // 10
            _ => unreachable!(),
        };

        let lib_path = vc_ltl5_dir
            .join("TargetPlatform")
            .join(win_ver)
            .join("lib")
            .join(arch);

        assert!(lib_path.exists());
        println!("cargo:rustc-link-search=native={}", lib_path.display());
    }

    // old windows compatibility
    if cfg!(feature = "win-xp") {
        let yy_thunks_dir = out_dir.join("YY-Thunks-Objs");

        if !yy_thunks_dir.exists() {
            let resp = minreq::get("https://github.com/Chuyu-Team/YY-Thunks/releases/download/v1.2.3/YY-Thunks-Objs.zip")
                .with_timeout(60)
                .send()
                .unwrap();
            assert_eq!(resp.status_code, 200);

            fs::create_dir_all(&yy_thunks_dir).unwrap();
            let mut cursor = std::io::Cursor::new(resp.as_bytes());
            ZipArchive::new(&mut cursor)
                .unwrap()
                .extract(&yy_thunks_dir)
                .unwrap();
        }

        let (obj_filename, arch, win_ver) = match target_arch.as_str() {
            "x86_64" => ("YY_Thunks_for_WinXP.obj", "x64", "5.02"),
            "x86" => ("YY_Thunks_for_WinXP.obj", "x86", "5.01"),
            _ => unreachable!(),
        };

        let obj_path = yy_thunks_dir.join("objs").join(arch).join(obj_filename);

        assert!(obj_path.exists());
        println!("cargo:rustc-link-arg={}", obj_path.display());
        println!("cargo:rustc-link-lib=dylib=ole32");
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS,{win_ver}");
    }

    if cfg!(feature = "static-vcrt") || cfg!(feature = "win-xp") {
        println!("cargo:rustc-link-arg=/VERBOSE:LIB");
    }

    // Generate Windows resources
    if target_os == "windows" {
        let mut res = WindowsResource::new();
        res.set_icon("package/windows/icon.ico");
        res.set_manifest_file("package/windows/TinyWiiBackupManager.exe.manifest");
        res.compile().unwrap();
    }
}
