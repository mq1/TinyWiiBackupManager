// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::Config, errors::Error};
use iced::Task;
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

#[derive(Debug)]
pub struct ToolboxItem {
    pub label: &'static str,
    pub run: fn(ToolContext) -> Task<Result<String, Error>>,
}

pub struct ToolboxGroup {
    pub label: &'static str,
    pub icon: Icon,
    pub items: &'static [ToolboxItem],
}

pub fn all() -> impl Iterator<Item = &'static ToolboxGroup> {
    [game_backup_loaders::ALL, cleanup::ALL, os_specific::ALL]
        .into_iter()
        .flatten()
}
