// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use eframe::egui;

/// State for console type filtering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsoleFilter {
    ShowAll,
    ShowWii,
    ShowGameCube,
    ShowNone,
}

impl Default for ConsoleFilter {
    fn default() -> Self {
        Self::ShowAll
    }
}

impl ConsoleFilter {
    /// Returns whether a Wii game should be shown
    pub fn shows_wii(&self) -> bool {
        matches!(self, ConsoleFilter::ShowAll | ConsoleFilter::ShowWii)
    }

    /// Returns whether a GameCube game should be shown
    pub fn shows_gc(&self) -> bool {
        matches!(self, ConsoleFilter::ShowAll | ConsoleFilter::ShowGameCube)
    }

    /// Returns whether a game should be shown based on its type
    pub fn shows_game(&self, is_gc: bool) -> bool {
        match (self, is_gc) {
            (ConsoleFilter::ShowAll, _) => true,
            (ConsoleFilter::ShowWii, false) => true,
            (ConsoleFilter::ShowGameCube, true) => true,
            _ => false,
        }
    }

    /// Toggles the Wii filter
    pub fn toggle_wii(&mut self) {
        *self = match self {
            ConsoleFilter::ShowAll => ConsoleFilter::ShowGameCube,
            ConsoleFilter::ShowWii => ConsoleFilter::ShowNone,
            ConsoleFilter::ShowGameCube => ConsoleFilter::ShowAll,
            ConsoleFilter::ShowNone => ConsoleFilter::ShowWii,
        };
    }

    /// Toggles the GameCube filter
    pub fn toggle_gc(&mut self) {
        *self = match self {
            ConsoleFilter::ShowAll => ConsoleFilter::ShowWii,
            ConsoleFilter::ShowWii => ConsoleFilter::ShowAll,
            ConsoleFilter::ShowGameCube => ConsoleFilter::ShowNone,
            ConsoleFilter::ShowNone => ConsoleFilter::ShowGameCube,
        };
    }
}

/// Renders the console filter controls
pub fn ui_console_filter(ui: &mut egui::Ui, filter: &mut ConsoleFilter) {
    let mut show_wii = filter.shows_wii();
    let mut show_gc = filter.shows_gc();

    ui.horizontal(|ui| {
        let wii_checkbox = ui.checkbox(&mut show_wii, "🎾 Show Wii");
        let gc_checkbox = ui.checkbox(&mut show_gc, "🎮 Show GC");

        // Update the filter based on checkbox changes
        if wii_checkbox.changed() || gc_checkbox.changed() {
            *filter = match (show_wii, show_gc) {
                (true, true) => ConsoleFilter::ShowAll,
                (true, false) => ConsoleFilter::ShowWii,
                (false, true) => ConsoleFilter::ShowGameCube,
                (false, false) => ConsoleFilter::ShowNone,
            };
        }
    });
}
