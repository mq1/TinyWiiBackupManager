// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleFilter {
    pub wii: bool,
    pub gamecube: bool,
}

impl Default for ConsoleFilter {
    fn default() -> Self {
        Self {
            wii: true,
            gamecube: true,
        }
    }
}

impl ConsoleFilter {
    /// Returns whether a game should be shown based on its type
    pub fn shows_game(&self, is_gc: bool) -> bool {
        if is_gc { self.gamecube } else { self.wii }
    }
}

/// Renders the console filter controls
pub fn ui_console_filter(ui: &mut egui::Ui, filter: &mut ConsoleFilter) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut filter.wii, "🎾 Show Wii");
        ui.checkbox(&mut filter.gamecube, "🎮 Show GC");
    });
}
