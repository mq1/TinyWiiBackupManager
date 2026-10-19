// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, fs, path::PathBuf, process::Command, thread};
use winresource::WindowsResource;

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
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let wii_dat_path = "assets/Nintendo - Wii - Datfile (3780) (2026-06-15 03-13-28).dat";
    let ngc_dat_path = "assets/Nintendo - GameCube - Datfile (2019) (2026-06-13 18-14-01).dat";

    println!("cargo:rerun-if-changed={wii_dat_path}");
    println!("cargo:rerun-if-changed={ngc_dat_path}");
    println!("cargo:rerun-if-changed=build.rs");

    let wii_hashes_handle = thread::spawn(|| extract_sha1_list(wii_dat_path));
    let ngc_hashes = extract_sha1_list(ngc_dat_path);
    let wii_hashes = wii_hashes_handle.join().unwrap();

    let mut sha1_list = wii_hashes;
    sha1_list.extend(ngc_hashes);
    sha1_list.sort_unstable();
    sha1_list.dedup();

    fs::write(out_dir.join("sha1_list.bin"), sha1_list.as_flattened()).unwrap();

    // lighter and static vcruntime replacement
    if cfg!(feature = "vc-ltl5") {
        let vc_ltl5_dir = out_dir.join("VC-LTL-Binary");

        if !vc_ltl5_dir.exists() {
            let archive_path = out_dir.join("VC-LTL-Binary.7z");

            let status = Command::new("curl.exe")
                .arg("-sSLo")
                .arg(&archive_path)
                .arg("https://github.com/Chuyu-Team/VC-LTL5/releases/download/v5.3.1/VC-LTL-Binary.7z")
                .status()
                .unwrap();
            assert!(status.success());

            let status = Command::new("tar.exe")
                .arg("-xf")
                .arg(&archive_path)
                .arg("-C")
                .arg(&vc_ltl5_dir)
                .status()
                .unwrap();
            assert!(status.success());
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
    if cfg!(feature = "yy-thunks") && target_arch != "aarch64" {
        let yy_thunks_dir = out_dir.join("YY-Thunks-Objs");

        if !yy_thunks_dir.exists() {
            let archive_path = out_dir.join("YY-Thunks-Objs.zip");

            let status = Command::new("curl.exe")
                .arg("-sSLo")
                .arg(&archive_path)
                .arg("https://github.com/Chuyu-Team/YY-Thunks/releases/download/v1.2.3/YY-Thunks-Objs.zip")
                .status()
                .unwrap();
            assert!(status.success());

            let status = Command::new("tar.exe")
                .arg("-xf")
                .arg(&archive_path)
                .arg("-C")
                .arg(&yy_thunks_dir)
                .status()
                .unwrap();
            assert!(status.success());
        }

        let (os, arch, win_ver) = match target_arch.as_str() {
            "x86_64" => ("Vista", "x64", "6.00"),
            "x86" => ("WinXP", "x86", "5.01"),
            _ => unreachable!(),
        };

        let obj_path = yy_thunks_dir
            .join("objs")
            .join(arch)
            .join(format!("YY_Thunks_for_{os}.obj"));

        assert!(obj_path.exists());
        println!("cargo:rustc-link-arg={}", obj_path.display());
        println!("cargo:rustc-link-lib=dylib=ole32");
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS,{win_ver}");
    }

    if cfg!(feature = "vc-ltl5") || cfg!(feature = "yy-thunks") {
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
