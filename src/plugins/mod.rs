// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod plugin;
mod tool;

use crate::plugins::plugin::Plugin;
use anyhow::{Result, anyhow};
use marwood::vm::Vm;
use std::{ffi::OsStr, fs, path::Path};

pub fn load(data_dir: impl AsRef<Path>) -> Result<Vec<Plugin>> {
    let plugins_dir = data_dir.as_ref().join("plugins");

    if !plugins_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut vm = Vm::new();
    let mut plugins = Vec::new();

    for entry in fs::read_dir(plugins_dir)?.filter_map(Result::ok) {
        let path = entry.path();

        let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
            continue;
        };

        let Some(ext) = path.extension().and_then(OsStr::to_str) else {
            continue;
        };

        if !path.is_file() || stem.starts_with('.') || ext != "scm" {
            continue;
        }

        println!("loading plugin: {}", path.display());

        let code = fs::read_to_string(&path)?;

        let (plugin_cell, remaining) = vm
            .eval_text(&code)
            .map_err(|_| anyhow!("Failed to evaluate plugin: {}", path.display()))?;

        println!("remaining: {:?}", remaining);

        let mut plugin = Plugin::try_from(&plugin_cell)?;

        plugin.id = stem.to_string();
        plugin.path = path;

        plugins.push(plugin);
    }

    Ok(plugins)
}
