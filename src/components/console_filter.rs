// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use bitflags::bitflags;
use eframe::egui;

bitflags! {
    /// State for console type filtering using bitflags.
    // We remove `Default` from the derive list to provide a custom implementation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ConsoleFilter: u8 {
        const WII = 1 << 0;
        const GAMECUBE = 1 << 1;

        /// A constant representing all filters being enabled.
        const ALL = Self::WII.bits() | Self::GAMECUBE.bits();
    }
}

/// Implements the default state for ConsoleFilter.
/// By default, all filters should be enabled.
impl Default for ConsoleFilter {
    fn default() -> Self {
        Self::ALL
    }
}

impl ConsoleFilter {
    /// Returns whether a game should be shown based on its type
    pub fn shows_game(&self, is_gc: bool) -> bool {
        if is_gc {
            self.contains(ConsoleFilter::GAMECUBE)
        } else {
            self.contains(ConsoleFilter::WII)
        }
    }
}

/// Renders the console filter controls
pub fn ui_console_filter(ui: &mut egui::Ui, filter: &mut ConsoleFilter) {
    // Create temporary boolean representations for the checkboxes
    let mut show_wii = filter.contains(ConsoleFilter::WII);
    let mut show_gc = filter.contains(ConsoleFilter::GAMECUBE);

    ui.horizontal(|ui| {
        // When a checkbox is changed, toggle the corresponding flag in the filter
        if ui.checkbox(&mut show_wii, "🎾 Show Wii").changed() {
            filter.toggle(ConsoleFilter::WII);
        }
        if ui.checkbox(&mut show_gc, "🎮 Show GC").changed() {
            filter.toggle(ConsoleFilter::GAMECUBE);
        }
    });
}
