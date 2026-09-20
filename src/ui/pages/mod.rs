// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use strum_macros::Display;

pub mod about;
pub mod errored;
pub mod games;
pub mod homebrew_apps;
pub mod import_queue;
pub mod loading;
pub mod osc_apps;
pub mod settings;
pub mod toolbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Display)]
pub enum Page {
    #[default]
    Games,
    #[strum(serialize = "Homebrew Apps")]
    HomebrewApps,
    #[strum(serialize = "Open Shop Channel")]
    Osc,
    Settings,
    Toolbox,
    #[strum(serialize = "Import Queue")]
    ImportQueue,
    About,
}
