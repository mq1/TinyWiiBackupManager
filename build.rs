// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, env, path::PathBuf};

fn main() {
    let library = HashMap::from([("lucide".to_string(), PathBuf::from(lucide_slint::lib()))]);
    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(library)
        .with_style("cupertino".into());

    slint_build::compile_with_config("ui/app_window.slint", config).unwrap();

    let target = env::var("TARGET").unwrap();

    if target.contains("-windows-") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("package/windows/icon.ico");
        if target.contains("-pc-") {
            res.set_manifest_file("package/windows/TinyWiiBackupManager.exe.manifest");
        }
        res.compile().unwrap();
    }
}
