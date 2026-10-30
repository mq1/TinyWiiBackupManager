// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use strum_macros::{Display, IntoStaticStr};

pub mod about;
pub mod consoles;
pub mod errored;
pub mod games;
pub mod homebrew_apps;
pub mod library;
pub mod loading;
pub mod no_drive;
pub mod osc_apps;
pub mod tasks;
pub mod toolbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, IntoStaticStr)]
pub enum Page {
    Library,
    Games,
    #[strum(serialize = "My Consoles")]
    Consoles,
    #[strum(serialize = "Homebrew Apps")]
    HomebrewApps,
    #[strum(serialize = "Open Shop Channel")]
    Osc,
    Settings,
    Toolbox,
    Tasks,
    About,
}
