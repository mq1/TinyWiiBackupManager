// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    http_util,
    notifications::{Notification, NotificationLevel},
};
use anyhow::Result;
use getset::{CloneGetters, Getters, WithSetters};
use mlua::{Lua, LuaSerdeExt, ObjectLike, Table, Value};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, WithSetters)]
#[serde(rename_all = "camelCase")]
pub struct PluginEnvironment {
    #[getset(set_with = "pub")]
    data_dir: String,

    #[getset(set_with = "pub")]
    mount_point: String,
}

#[derive(Debug, Clone, Deserialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct PluginMeta {
    #[getset(get = "pub")]
    name: String,

    #[getset(get = "pub")]
    description: String,

    #[getset(get = "pub")]
    version: String,

    #[getset(get = "pub")]
    authors: Vec<String>,

    #[getset(get = "pub")]
    license: String,

    #[getset(get = "pub")]
    runs_on: Vec<String>,
}

#[derive(Debug, Clone, Getters, CloneGetters)]
pub struct Plugin {
    #[getset(get = "pub")]
    path: PathBuf,

    #[getset(get = "pub")]
    meta: PluginMeta,

    code: String,
}

impl Plugin {
    pub fn load(path: PathBuf) -> Result<Self> {
        let lua = Lua::new();
        let code = fs::read_to_string(&path)?;
        let plugin = lua.load(&code).eval::<Table>()?;

        let meta = plugin.get("meta")?;
        let meta = lua.from_value(meta)?;

        Ok(Self { path, meta, code })
    }

    pub fn can_run(&self) -> bool {
        self.meta.runs_on.iter().any(|s| s == std::env::consts::OS)
    }

    pub fn run(&self, env: &PluginEnvironment) -> Result<Option<Notification>> {
        let lua = Lua::new();
        let plugin = lua.load(&self.code).eval::<Table>()?;

        // Register handy functions
        let globals = lua.globals();

        let download_file = lua.create_function(|_, (uri, dest): (String, String)| {
            http_util::download_file(&uri, &dest).map_err(Into::into)
        })?;
        globals.set("downloadFile", download_file)?;

        let env = lua.to_value(env)?;
        let res = plugin.call_function::<Value>("run", env)?;

        if res.is_nil() {
            return Ok(None);
        }

        if let Ok(label) = res.to_string() {
            return Ok(Some(Notification::new(label, NotificationLevel::Info)));
        }

        Ok(Some(lua.from_value(res)?))
    }
}

pub fn list(data_dir: &Path) -> Result<Vec<Plugin>> {
    let plugins_dir = data_dir.join("plugins");

    let paths = fs::read_dir(plugins_dir)?.filter_map(|entry| {
        let path = entry.ok()?.path();
        let stem = path.file_stem()?.to_str()?;
        let ext = path.extension()?.to_str()?;
        (path.is_file() && !stem.starts_with('.') && ext == "lua").then_some(path)
    });

    paths.map(Plugin::load).collect()
}
