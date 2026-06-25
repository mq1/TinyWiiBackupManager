// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use marwood::cell::Cell;

#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub group: String,
    pub run: Cell,
}

impl TryFrom<&Cell> for Tool {
    type Error = anyhow::Error;

    fn try_from(value: &Cell) -> Result<Self> {
        let mut name = None;
        let mut description = None;
        let mut icon = None;
        let mut group = None;
        let mut run = None;

        for v in value {
            let Some(k) = v.car().and_then(Cell::as_symbol) else {
                continue;
            };
            let Some(v) = v.cdr() else { continue };

            match k {
                "name" => name = Some(v.to_string()),
                "description" => description = Some(v.to_string()),
                "icon" => icon = Some(v.to_string()),
                "group" => group = Some(v.to_string()),
                "run" => run = Some(v.clone()),
                _ => {}
            }
        }

        let name = name.ok_or_else(|| anyhow!("name not found"))?;
        let description = description.ok_or_else(|| anyhow!("description not found"))?;
        let icon = icon.ok_or_else(|| anyhow!("icon not found"))?;
        let group = group.ok_or_else(|| anyhow!("group not found"))?;
        let run = run.ok_or_else(|| anyhow!("run not found"))?;

        Ok(Self {
            name,
            description,
            icon,
            group,
            run,
        })
    }
}
