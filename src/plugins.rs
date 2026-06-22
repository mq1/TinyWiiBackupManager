// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::notifications::{Notification, NotificationLevel};
use anyhow::{Result, anyhow};
use getset::{CloneGetters, Getters, WithSetters};
use mlua::{Lua, ObjectLike, Table, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Default, WithSetters)]
pub struct PluginEnvironment {
    #[getset(set_with = "pub")]
    data_dir: String,

    #[getset(set_with = "pub")]
    mount_point: String,
}

impl PluginEnvironment {
    pub fn into_table(self, lua: &Lua) -> Result<Table> {
        let table = lua.create_table()?;
        table.set("dataDir", self.data_dir)?;
        table.set("mountPoint", self.mount_point)?;
        Ok(table)
    }
}

#[derive(Debug, Clone, Getters)]
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

        let meta = plugin.get::<Table>("meta")?;
        let name = meta.get("name")?;
        let version = meta.get("version")?;
        let authors = meta.get("authors")?;
        let description = meta.get("description")?;
        let license = meta.get("license")?;
        let runs_on = meta.get("runsOn")?;

        let meta = PluginMeta {
            name,
            description,
            version,
            authors,
            license,
            runs_on,
        };

        Ok(Self { path, meta, code })
    }

    pub fn can_run(&self) -> bool {
        self.meta.runs_on.iter().any(|s| s == std::env::consts::OS)
    }

    pub fn run(&self, environment: PluginEnvironment) -> Result<Option<Notification>> {
        let lua = Lua::new();
        let plugin = lua.load(&self.code).eval::<Table>()?;

        let environment = environment.into_table(&lua)?;
        let res = plugin.call_function::<Value>("run", environment)?;

        match res {
            Value::Nil => Ok(None),
            Value::String(label) => Ok(Some(Notification::new(
                label.to_string_lossy(),
                NotificationLevel::Info,
            ))),
            Value::Table(res) => {
                let label = res.get::<String>("label")?;
                let level = res.get::<String>("level")?.parse()?;
                Ok(Some(Notification::new(label, level)))
            }
            _ => Err(anyhow!("Invalid return value from plugin")),
        }
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
