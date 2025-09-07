// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, AsRefStr, EnumIter, Copy)]
pub enum WiiOutputFormat {
    #[default]
    #[strum(serialize = "✨ WBFS Auto Split (recommended)")]
    WbfsAuto,
    #[strum(serialize = "📏 WBFS Fixed 4GB-32KB Split Size")]
    WbfsFixed,
    #[strum(serialize = "💿 ISO (very large)")]
    Iso,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, AsRefStr, EnumIter, Copy)]
pub enum StripPartitions {
    #[default]
    #[strum(serialize = "🛡 Keep all (recommended)")]
    No,
    #[strum(serialize = "🗑 Remove Update (integrity check disabled)")]
    Update,
    #[strum(serialize = "⚠ Remove all but Game (integrity check disabled, not recommended)")]
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub wii_output_format: WiiOutputFormat,
    pub strip_partitions: StripPartitions,
    #[serde(default = "default_wii_ip")]
    pub wii_ip: String,
}

fn default_wii_ip() -> String {
    "192.168.1.100".to_string()
}
