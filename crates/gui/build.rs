// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{env, fs, path::PathBuf};
use winresource::WindowsResource;
use zip::ZipArchive;

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();

    println!("cargo:rerun-if-changed=build.rs");

    // lighter and static vcruntime replacement
    if cfg!(feature = "dist-windows") && target_env == "msvc" {
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
            "x86_64" => ("6.0.6000.0", "x64"),      // vista+
            "x86" => ("6.0.6000.0", "Win32"),       // vista+
            "aarch64" => ("10.0.10240.0", "ARM64"), // 10+
            _ => unreachable!(),
        };

        let lib_path = vc_ltl5_dir
            .join("TargetPlatform")
            .join(win_ver)
            .join("lib")
            .join(arch);
        assert!(lib_path.exists());

        println!("cargo:rustc-link-arg=/LIBPATH:{}", lib_path.display());
        println!("cargo:rustc-link-search=native={}", lib_path.display());
    }

    // windows 7 compatibility
    if cfg!(feature = "dist-windows")
        && target_env == "msvc"
        && (target_arch == "x86_64" || target_arch == "x86")
    {
        let yy_thunks_dir = out_dir.join("YY-Thunks-Lib");

        if !yy_thunks_dir.exists() {
            let resp = minreq::get("https://github.com/Chuyu-Team/YY-Thunks/releases/download/v1.2.4-Beta.1/YY-Thunks-Lib.zip")
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

        let arch = match target_arch.as_str() {
            "x86_64" => "x64",
            "x86" => "x86",
            _ => unreachable!(),
        };

        let lib_path = yy_thunks_dir.join("Lib").join("6.1.7600.0").join(arch);
        assert!(lib_path.exists());

        println!("cargo:rustc-link-arg=/LIBPATH:{}", lib_path.display());
        println!("cargo:rustc-link-search=native={}", lib_path.display());
        println!("cargo:rustc-link-arg=/DELAYLOAD:api-ms-win-core-winrt-string-l1-1-0.dll");
        println!("cargo:rustc-link-lib=delayimp");
        println!("cargo:rustc-link-lib=ole32");
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS,6.1");
    }

    // Generate Windows resources
    if target_os == "windows" {
        let mut res = WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set_manifest_file("assets/TinyWiiBackupManager.exe.manifest");
        res.compile().unwrap();
    }
}
