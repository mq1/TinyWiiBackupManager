// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod about;
pub mod errored;
pub mod games;
pub mod homebrew_apps;
pub mod import_queue;
pub mod loading;
pub mod no_drive;
pub mod osc_apps;
pub mod toolbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Games,
    HomebrewApps,
    Osc,
    Settings,
    Toolbox,
    ImportQueue,
    About,
}

impl Page {
    pub fn as_str(&self) -> &str {
        match self {
            Page::Games => "Games",
            Page::HomebrewApps => "Homebrew Apps",
            Page::Osc => "Open Shop Channel",
            Page::Settings => "Settings",
            Page::Toolbox => "Toolbox",
            Page::ImportQueue => "Import Queue",
            Page::About => "About",
        }
    }
}
