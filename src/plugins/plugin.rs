// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::plugins::tool::Tool;
use anyhow::{Result, anyhow};
use marwood::cell::Cell;
use std::path::PathBuf;

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

impl TryFrom<&Cell> for Plugin {
    type Error = anyhow::Error;

    fn try_from(value: &Cell) -> Result<Self> {
        let mut name = None;
        let mut version = None;
        let mut authors = None;
        let mut description = None;
        let mut license = None;
        let mut runs_on = None;
        let mut tools = None;

        for v in value {
            let Some(k) = v.car().and_then(Cell::as_symbol) else {
                continue;
            };
            let Some(v) = v.cdr() else {
                continue;
            };

            match (k, v) {
                ("name", v) => name = Some(v.to_string()),
                ("version", v) => version = Some(v.to_string()),
                ("authors", v) => authors = Some(v.iter().map(Cell::to_string).collect()),
                ("description", v) => description = Some(v.to_string()),
                ("license", v) => license = Some(v.to_string()),
                ("runs-on", v) => runs_on = Some(v.iter().map(Cell::to_string).collect()),
                ("tools", v) => {
                    tools = Some(v.iter().map(Tool::try_from).collect::<Result<Vec<_>>>()?)
                }
                _ => {}
            }
        }

        let name = name.ok_or_else(|| anyhow!("name is required"))?;
        let version = version.ok_or_else(|| anyhow!("version is required"))?;
        let authors = authors.ok_or_else(|| anyhow!("authors is required"))?;
        let description = description.ok_or_else(|| anyhow!("description is required"))?;
        let license = license.ok_or_else(|| anyhow!("license is required"))?;
        let runs_on = runs_on.ok_or_else(|| anyhow!("runs_on is required"))?;
        let tools = tools.ok_or_else(|| anyhow!("tools is required"))?;

        Ok(Plugin {
            id: String::new(),
            path: PathBuf::new(),
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
