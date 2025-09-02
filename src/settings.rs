// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, AsRefStr, EnumIter, Copy)]
pub enum OutputFormat {
    #[default]
    #[strum(serialize = "✨ WBFS Auto Split (recommended)")]
    WbfsAuto,
    #[strum(serialize = "📏 WBFS Fixed 4GB-32KB Split Size")]
    WbfsFixed,
    #[strum(serialize = "💿 ISO (very large)")]
    Iso,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub output_format: OutputFormat,
}
