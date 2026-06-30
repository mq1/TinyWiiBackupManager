// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Plugin {
    pub path: PathBuf,
    pub meta: PluginMeta,
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub description: String,
    pub license: String,
    pub runs_on: Vec<String>,
    pub tools: Vec<ToolMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolMeta {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub group: String,
}
