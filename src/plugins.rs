// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::notifications::{Notification, NotificationLevel};
use anyhow::{Result, anyhow};
use getset::{CloneGetters, Getters, WithSetters};
use std::{
    fs,
    path::{Path, PathBuf},
};
use steel::steel_vm::engine::Engine;

#[derive(Debug, Clone, Default, WithSetters)]
pub struct PluginEnvironment {
    #[getset(set_with = "pub")]
    data_dir: String,

    #[getset(set_with = "pub")]
    mount_point: String,
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
        let mut vm = Engine::new_base();
        let code = fs::read_to_string(&path)?;
        let _ = vm.run(code.clone())?;

        let name = vm.extract("name")?;
        let version = vm.extract("version")?;
        let authors = vm.extract("authors")?;
        let description = vm.extract("description")?;
        let license = vm.extract("license")?;
        let runs_on = vm.extract("runs-on")?;

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
        let mut vm = Engine::new();
        vm.register_value("twbm/data-dir", environment.data_dir.into());
        vm.register_value("twbm/mount-point", environment.mount_point.into());

        let _ = vm.run(self.code.clone())?;
        let res = vm.call_function_by_name_with_args_from_mut_slice("run", &mut [])?;
        let res = res.list_or_else(|| anyhow!("invalid return value"))?;

        let Some(label) = res.get(0) else {
            return Ok(None);
        };
        let label = label
            .as_string()
            .ok_or_else(|| anyhow!("invalid notification label"))?;

        let Some(level) = res.get(1) else {
            return Ok(Some(Notification::new(
                label.as_str(),
                NotificationLevel::Info,
            )));
        };
        let level = level
            .as_string()
            .and_then(|l| l.parse().ok())
            .ok_or_else(|| anyhow!("invalid notification level"))?;

        Ok(Some(Notification::new(label.as_str(), level)))
    }
}

pub fn list(data_dir: &Path) -> Result<Vec<Plugin>> {
    let plugins_dir = data_dir.join("plugins");

    let paths = fs::read_dir(plugins_dir)?.filter_map(|entry| {
        let path = entry.ok()?.path();
        let stem = path.file_stem()?.to_str()?;
        let ext = path.extension()?.to_str()?;
        (path.is_file() && !stem.starts_with('.') && ext == "scm").then_some(path)
    });

    paths.map(Plugin::load).collect()
}
