// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, path::Path};

fn main() {
    let config = slint_build::CompilerConfiguration::new().with_library_paths(HashMap::from([(
        "material".to_string(),
        Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .join("material-1.0/material.slint"),
    )]));
    slint_build::compile_with_config("ui/main.slint", config).unwrap();

    // Windows-specific icon resource
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/logo.ico");
        res.compile().unwrap();
    }
}
