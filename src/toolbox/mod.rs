// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::errors::Error;
use iced::Task;
use lucide_icons::Icon;
use std::path::PathBuf;

mod os_specific;

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub mount_point: PathBuf,
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
    os_specific::all()
}
