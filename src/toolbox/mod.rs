// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::config::Config;
use anyhow::Result;
use lucide_icons::Icon;
use smol_str::SmolStr;
use std::pin::Pin;
use wii_disc_info::game_id::GameID;

mod cleanup;
mod game_backup_loaders;
mod os_specific;

type RunFn = Pin<Box<dyn Future<Output = Result<SmolStr>> + Send>>;

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub config: Config,
    pub game_ids: Vec<GameID>,
}

#[derive(Debug)]
pub struct ToolboxItem {
    label: &'static str,
    run_fn: fn(ToolContext) -> RunFn,
}

impl ToolboxItem {
    pub fn label(&self) -> &'static str {
        self.label
    }

    pub async fn run(&self, context: ToolContext) -> Result<SmolStr> {
        (self.run_fn)(context).await
    }
}

pub struct ToolboxGroup {
    label: &'static str,
    icon: Icon,
    items: &'static [ToolboxItem],
}

impl ToolboxGroup {
    pub fn label(&self) -> &'static str {
        self.label
    }

    pub fn icon(&self) -> Icon {
        self.icon
    }

    pub fn items(&self) -> impl Iterator<Item = &'static ToolboxItem> {
        self.items.iter()
    }
}

pub fn all() -> impl Iterator<Item = &'static ToolboxGroup> {
    [game_backup_loaders::ALL, cleanup::ALL, os_specific::ALL]
        .into_iter()
        .flatten()
}
