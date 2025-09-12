// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, AsRefStr, EnumIter, Copy)]
pub enum WiiOutputFormat {
    #[strum(serialize = "✨ WBFS Auto Split (recommended)")]
    WbfsAuto,
    #[strum(serialize = "📏 WBFS Fixed 4GB-32KB Split Size")]
    WbfsFixed,
    #[strum(serialize = "💿 ISO (very large)")]
    Iso,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, AsRefStr, EnumIter, Copy)]
pub enum ArchiveFormat {
    #[strum(serialize = "📦 RVZ zstd-19 (recommended)")]
    Rvz,
    #[strum(serialize = "💿 ISO (very large)")]
    Iso,
}

impl ArchiveFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::Rvz => "rvz",
            ArchiveFormat::Iso => "iso",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, AsRefStr, EnumIter, Copy)]
pub enum StripPartitions {
    #[strum(serialize = "🛡 Keep all (recommended)")]
    No,
    #[strum(serialize = "🗑 Remove Update (integrity check disabled)")]
    Update,
    #[strum(serialize = "⚠ Remove all but Game (integrity check disabled, not recommended)")]
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub wii_output_format: WiiOutputFormat,
    pub archive_format: ArchiveFormat,
    pub strip_partitions: StripPartitions,
    pub wii_ip: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            wii_output_format: WiiOutputFormat::WbfsAuto,
            archive_format: ArchiveFormat::Rvz,
            strip_partitions: StripPartitions::No,
            wii_ip: "192.168.1.100".to_string(),
        }
    }
}
