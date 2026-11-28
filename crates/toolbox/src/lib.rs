// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use twbm_common::config_contents::ConfigContents;
use wii_disc_info::game_id::GameID;

mod cleanup;
mod game_backup_loaders;
mod os_specific;
pub mod txtcodes;

pub use game_backup_loaders::covers;

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub config: ConfigContents,
    pub game_ids: Box<[GameID]>,
}

#[derive(Debug)]
pub struct ToolboxItem {
    label: &'static str,
    run: fn(ToolContext) -> Result<String>,
}

impl ToolboxItem {
    pub fn label(&self) -> &'static str {
        self.label
    }

    pub fn run(&self, ctx: ToolContext) -> Result<String> {
        (self.run)(ctx)
    }
}

pub struct ToolboxGroup {
    label: &'static str,
    icon: &'static str,
    items: &'static [ToolboxItem],
}

impl ToolboxGroup {
    pub fn label(&self) -> &'static str {
        self.label
    }

    pub fn icon(&self) -> &'static str {
        self.icon
    }

    pub fn items(&self) -> impl Iterator<Item = &'static ToolboxItem> {
        self.items.iter()
    }
}

pub fn all() -> impl Iterator<Item = &'static ToolboxGroup> {
    game_backup_loaders::ALL
        .iter()
        .chain(txtcodes::ALL)
        .chain(cleanup::ALL)
        .chain(os_specific::ALL)
}
