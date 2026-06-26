// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::plugins::tool::Tool;
use mlua::{FromLua, Lua, Value};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Plugin {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
    pub license: String,
    pub runs_on: Vec<String>,
    pub tools: Vec<Tool>,
}

impl FromLua for Plugin {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        let Some(table) = value.as_table() else {
            return Err(mlua::Error::UserDataTypeMismatch);
        };

        let id = String::new();
        let path = PathBuf::new();

        let name = table.get("name")?;
        let version = table.get("version")?;
        let authors = table.get("authors")?;
        let description = table.get("description")?;
        let license = table.get("license")?;
        let runs_on = table.get("runs_on")?;
        let tools = table.get("tools")?;

        Ok(Plugin {
            id,
            path,
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
