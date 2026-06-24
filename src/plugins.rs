// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::state::AppState;
use anyhow::Result;
use mlua::{FromLua, Function, Lua, ObjectLike, Table};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct Plugin {
    id: String,
    path: PathBuf,
    plugin: Table,
}

#[derive(Debug, Clone, FromLua)]
pub struct Tool {
    name: String,
    decription: String,
    icon: String,
    group: String,
    run: Function,
}

pub fn load_all(data_dir: &Path) -> Result<Lua> {
    let plugins_dir = data_dir.join("plugins");
    let lua = Lua::new();

    let plugins_table = lua.create_table()?;
    for entry in fs::read_dir(plugins_dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
    {
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

        let code = fs::read_to_string(&path)?;
        let plugin = lua.load(&code).eval::<Table>()?;

        let runs_on = plugin.get::<Vec<String>>("runs_on")?;
        if !runs_on.iter().any(|os| os == std::env::consts::OS) {
            continue;
        }

        plugins_table.set(stem, plugin)?;
    }

    lua.globals().set("plugins", plugins_table)?;
    Ok(lua)
}

pub fn init_all(state: &mut AppState) -> Result<()> {
    let lua = &state.plugins;

    lua.scope(|scope| {
        let register_tool = scope.create_function_mut(|_, tool: Tool| {
            println!("{tool:?}");
            Ok(())
        })?;

        let notify = scope.create_function_mut(|_, msg: String| {
            state.notifications.add(msg);
            Ok(())
        })?;

        let twbm = lua.create_table()?;
        twbm.set("register_tool", register_tool)?;
        twbm.set("notify", notify)?;

        let plugins = lua.globals().get::<Table>("plugins")?;
        for plugin in plugins.sequence_values::<Table>() {
            plugin?.call_function::<Table>("init", &twbm)?;
        }

        Ok(())
    })?;

    Ok(())
}
