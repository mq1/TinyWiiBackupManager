// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use bitflags::bitflags;
use eframe::egui;

bitflags! {
    /// State for console type filtering using bitflags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ConsoleFilter: u8 {
        const WII = 1 << 0;
        const GAMECUBE = 1 << 1;
        // New flags can be added easily
        // const SWITCH = 1 << 2;

        const ALL = Self::WII.bits() | Self::GAMECUBE.bits();
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
    let mut show_wii = filter.contains(ConsoleFilter::WII);
    let mut show_gc = filter.contains(ConsoleFilter::GAMECUBE);

    ui.horizontal(|ui| {
        if ui.checkbox(&mut show_wii, "🎾 Show Wii").changed() {
            filter.toggle(ConsoleFilter::WII);
        }
        if ui.checkbox(&mut show_gc, "🎮 Show GC").changed() {
            filter.toggle(ConsoleFilter::GAMECUBE);
        }
    });
}
