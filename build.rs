// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

fn main() {
    // MacOS-specific settings
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=11");
    }

    // Windows-specifix fixes
    #[cfg(windows)]
    {
        println!("cargo:rustc-link-lib=kernel32");
    }

    // Windows-specific icon resource
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/TinyWiiBackupManager.ico");
        res.compile().unwrap();
    }
}
