// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::plugins::{cell_ext::CellExt, tool::Tool};
use anyhow::{Result, anyhow};
use marwood::{cell, cell::Cell, vm::Vm};
use std::{ffi::OsStr, fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Plugin {
    pub id: String,
    pub path: PathBuf,
    name: String,
    version: String,
    authors: Vec<String>,
    description: String,
    license: String,
    runs_on: Vec<String>,
    tools: Vec<Tool>,
}

impl TryFrom<PathBuf> for Plugin {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let id = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| anyhow!("Invalid plugin path: {}", path.display()))?;

        let code = fs::read_to_string(&path)?;
        let mut vm = Vm::new();

        let mut next = Some(&code[..]);
        while let Some(current) = next {
            (_, next) = vm
                .eval_text(current)
                .map_err(|_| anyhow!("Failed to evaluate plugin: {}", path.display()))?;
        }

        let name = vm
            .eval(&cell!("name"))
            .ok()
            .and_then(Cell::into_string)
            .ok_or_else(|| anyhow!("name is required"))?;
        let version = vm
            .eval(&cell!("version"))
            .ok()
            .and_then(Cell::into_string)
            .ok_or_else(|| anyhow!("version is required"))?;
        let authors = vm
            .eval(&cell!("authors"))
            .ok()
            .and_then(|c| c.into_iter().map(Cell::into_string).collect())
            .ok_or_else(|| anyhow!("authors is required"))?;
        let description = vm
            .eval(&cell!("description"))
            .ok()
            .and_then(Cell::into_string)
            .ok_or_else(|| anyhow!("description is required"))?;
        let license = vm
            .eval(&cell!("license"))
            .ok()
            .and_then(Cell::into_string)
            .ok_or_else(|| anyhow!("license is required"))?;
        let runs_on = vm
            .eval(&cell!("runs-on"))
            .ok()
            .and_then(|c| c.into_iter().map(Cell::into_string).collect())
            .ok_or_else(|| anyhow!("runs_on is required"))?;

        let tools = vm
            .eval(&cell!("tools"))
            .map_err(|_| anyhow!("tools is required"))?
            .into_iter()
            .map(|t| Tool::try_from(&t))
            .collect::<Result<Vec<Tool>, _>>()?;

        Ok(Plugin {
            id: id.to_string(),
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
