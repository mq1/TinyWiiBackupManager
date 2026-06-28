// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::plugins::tool::Tool;
use mlua::{FromLua, Lua, Value};
use std::{
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};

pub static LAST_ID: LazyLock<Arc<Mutex<u32>>> = LazyLock::new(|| Arc::new(Mutex::new(u32::MAX)));

#[derive(Debug, Clone)]
pub struct Plugin {
    pub path: PathBuf,
    pub contents: PluginContents,
}

#[derive(Debug, Clone)]
pub struct PluginContents {
    pub id: u32,
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
    pub license: String,
    pub runs_on: Vec<String>,
    pub tools: Vec<Tool>,
}

impl FromLua for PluginContents {
    fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
        let table = value
            .as_table()
            .ok_or(mlua::Error::runtime("expected a table"))?;

        let name = table.get("name")?;
        let version = table.get("version")?;
        let authors = table.get("authors")?;
        let description = table.get("description")?;
        let license = table.get("license")?;
        let runs_on = table.get("runs_on")?;
        let tools = table.get("tools")?;

        let id = {
            let mut last_id = LAST_ID.lock().unwrap();
            *last_id = last_id.wrapping_add(1);
            *last_id
        };

        Ok(PluginContents {
            id,
            name,
            version,
            authors,
            description,
            license,
            runs_on,
            tools,
        })
    }
}
