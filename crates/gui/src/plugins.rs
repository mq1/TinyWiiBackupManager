// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::Notification;
use anyhow::{Result, anyhow, bail};
use std::{fs, path::PathBuf};
use steel::steel_vm::engine::Engine;
use twbm_core::data_dir::DATA_DIR;

pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
    pub license: String,
    pub runs_on: Vec<String>,
}

pub struct Plugin {
    pub path: PathBuf,
    pub code: String,
    pub meta: PluginMeta,
}

impl Plugin {
    pub fn load(path: PathBuf) -> Result<Self> {
        let mut vm = Engine::new_base();
        let code = fs::read_to_string(&path)?;
        vm.run(code.clone())?;

        let name = vm.extract("name")?;
        let version = vm.extract("version")?;
        let authors = vm.extract("authors")?;
        let description = vm.extract("description")?;
        let license = vm.extract("license")?;
        let runs_on = vm.extract("runs-on")?;

        let meta = PluginMeta {
            name,
            version,
            authors,
            description,
            license,
            runs_on,
        };

        Ok(Self { path, code, meta })
    }

    pub fn can_run(&self) -> bool {
        self.meta.runs_on.iter().any(|s| s == std::env::consts::OS)
    }

    pub fn run(&self) -> Result<Notification> {
        let mut vm = Engine::new();

        let code = self.code.clone() + "(run)";
        let res = vm.run(code)?;

        if res.len() != 2 {
            bail!("run function returned {} values, expected 2", res.len());
        }

        let msg = res[0].to_string();
        let critical = res[1].bool_or_else(|| anyhow!("critical flag is not a boolean"))?;

        let notification = Notification {
            text: msg.into(),
            critical,
        };

        Ok(notification)
    }
}

pub fn list() -> impl Iterator<Item = Plugin> {
    let plugins_dir = DATA_DIR.join("plugins");

    let paths = fs::read_dir(plugins_dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let ext = path.extension()?.to_str()?;
            (path.is_file() && ext == "scm").then_some(path)
        });

    paths.filter_map(|path| match Plugin::load(path) {
        Ok(plugin) => {
            eprintln!("Loaded plugin: {}", plugin.meta.name);
            Some(plugin)
        }
        Err(e) => {
            eprintln!("Failed to load plugin: {e}");
            None
        }
    })
}
