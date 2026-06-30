// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod plugin;
pub mod run;

use crate::plugins::plugin::Plugin;
use anyhow::Result;
use mlua::{Lua, LuaSerdeExt};
use smol::{fs, stream::StreamExt};
use std::{ffi::OsStr, path::Path};

pub async fn load(data_dir: impl AsRef<Path>) -> Result<Vec<Plugin>> {
    let plugins_dir = data_dir.as_ref().join("plugins");
    let mut plugins = Vec::new();

    if !fs::metadata(&plugins_dir)
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        return Ok(plugins);
    }

    let lua = Lua::new();

    while let Some(entry) = fs::read_dir(&plugins_dir).await?.next().await {
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

        if !path.is_file() || stem.starts_with('.') || ext != "lua" {
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
