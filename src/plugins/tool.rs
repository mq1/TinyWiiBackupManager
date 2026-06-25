// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use mlua::{FromLua, Function};

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub group: String,
    pub run: Function,
}

impl FromLua for Tool {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        let Some(table) = value.as_table() else {
            return Err(mlua::Error::UserDataTypeMismatch);
        };

        let name = table.get("name")?;
        let description = table.get("description")?;
        let icon = table.get("icon")?;
        let group = table.get("group")?;
        let run = table.get("run")?;

        Ok(Tool {
            name,
            description,
            icon,
            group,
            run,
        })
    }
}
