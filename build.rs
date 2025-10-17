// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use slint_build::EmbedResourcesKind;
use std::{collections::HashMap, env::var, path::Path};

fn main() {
    let manifest_dir = var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env var not set");

    let material_path = Path::new(&manifest_dir).join("material-1.0/material.slint");

    let mut library_paths = HashMap::new();
    library_paths.insert("material".to_string(), material_path);

    let config = slint_build::CompilerConfiguration::new()
        .with_library_paths(library_paths)
        .with_style("material".into())
        .embed_resources(EmbedResourcesKind::EmbedFiles);

    slint_build::compile_with_config("ui/main.slint", config).unwrap();

    // Windows-specific icon resource
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/logo.ico");
        res.compile().unwrap();
    }
}
