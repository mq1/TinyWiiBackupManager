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
            let Cell::Pair(k, v) = v else { continue };

            match (k.as_symbol(), v.as_ref()) {
                (Some("name"), Cell::String(v)) => name = Some(v.clone()),
                (Some("version"), Cell::String(v)) => version = Some(v.clone()),
                (Some("authors"), v) => {
                    authors = v
                        .iter()
                        .map(|v| {
                            if let Cell::String(v) = v {
                                Some(v.clone())
                            } else {
                                None
                            }
                        })
                        .collect()
                }
                (Some("description"), Cell::String(v)) => description = Some(v.clone()),
                (Some("license"), Cell::String(v)) => license = Some(v.clone()),
                (Some("runs-on"), v) => {
                    runs_on = v
                        .iter()
                        .map(|v| {
                            if let Cell::String(v) = v {
                                Some(v.clone())
                            } else {
                                None
                            }
                        })
                        .collect()
                }
                (Some("tools"), v) => {
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
