// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::Config, errors::Error};
use async_trait::async_trait;
use lucide_icons::Icon;
use wii_disc_info::game_id::GameID;

mod cleanup;
mod game_backup_loaders;
mod os_specific;

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub config: Config,
    pub game_ids: Vec<GameID>,
}

#[async_trait]
pub trait ToolboxItem: std::fmt::Debug + Send + Sync {
    fn label(&self) -> &'static str;
    async fn run(&self, ctx: ToolContext) -> Result<String, Error>;
}

pub struct ToolboxGroup {
    pub label: &'static str,
    pub icon: Icon,
    pub items: &'static [&'static dyn ToolboxItem],
}

pub fn all() -> impl Iterator<Item = &'static ToolboxGroup> {
    [game_backup_loaders::ALL, cleanup::ALL, os_specific::ALL]
        .into_iter()
        .flatten()
}
