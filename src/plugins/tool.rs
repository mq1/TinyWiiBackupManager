// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ToolMeta {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub group: String,
}
