// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod plugin;
pub mod run;

use crate::plugins::plugin::Plugin;
use anyhow::Result;
use mlua::{Lua, LuaSerdeExt};
use smol::{fs, stream::StreamExt};
use std::{ffi::OsStr, path::PathBuf};

pub async fn load(data_dir: impl Into<PathBuf>) -> Result<Vec<Plugin>> {
    let mut plugins_dir = data_dir.into();
    plugins_dir.push("plugins");

    let mut plugins = Vec::new();

    if !fs::metadata(&plugins_dir).await.is_ok_and(|m| m.is_dir()) {
        return Ok(plugins);
    }

    let lua = Lua::new();

    let mut entries = fs::read_dir(&plugins_dir).await?;
    while let Some(entry) = entries.next().await {
        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();

        let Some(stem) = path.file_stem().and_then(OsStr::to_str) else {
            continue;
        };

        let Some(ext) = path.extension().and_then(OsStr::to_str) else {
            continue;
        };

        if stem.starts_with('.') || ext != "lua" {
            continue;
        }

        let code = fs::read_to_string(&path).await?;
        let meta_table = lua.load(&code).eval()?;

        let meta = lua.from_value_with(
            meta_table,
            mlua::serde::de::Options::new().deny_unsupported_types(false),
        )?;

        let plugin = Plugin { path, meta, code };

        plugins.push(plugin);
    }

    Ok(plugins)
}
