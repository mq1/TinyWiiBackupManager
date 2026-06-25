// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::plugins::cell_ext::CellExt;
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
        let name = value
            .get_alist_value("name")
            .and_then(Cell::as_string)
            .ok_or_else(|| anyhow!("name is required"))?;
        let description = value
            .get_alist_value("description")
            .and_then(Cell::as_string)
            .unwrap_or_default();
        let icon = value
            .get_alist_value("icon")
            .and_then(Cell::as_string)
            .unwrap_or_default();
        let group = value
            .get_alist_value("group")
            .and_then(Cell::as_string)
            .unwrap_or_default();
        let run = value
            .get_alist_value("run")
            .ok_or_else(|| anyhow!("run is required"))?;

        Ok(Self {
            name: name.to_string(),
            description: description.to_string(),
            icon: icon.to_string(),
            group: group.to_string(),
            run: run.clone(),
        })
    }
}
