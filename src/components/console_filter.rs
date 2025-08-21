// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use eframe::egui;

/// State for console type filtering
pub struct ConsoleFilter {
    pub show_wii: bool,
    pub show_gc: bool,
}

impl Default for ConsoleFilter {
    fn default() -> Self {
        Self {
            show_wii: true,
            show_gc: true,
        }
    }
}

/// Renders the console filter controls
pub fn ui_console_filter(ui: &mut egui::Ui, filter: &mut ConsoleFilter) {
    ui.horizontal(|ui| {
        ui.checkbox(&mut filter.show_wii, "🎾 Show Wii");
        ui.checkbox(&mut filter.show_gc, "🎮 Show GC");
    });
}
