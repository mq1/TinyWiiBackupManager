// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::{ConsoleType, Game};
use eframe::egui;
use strum::{EnumMessage, IntoEnumIterator};

pub struct ConsoleFilter {
    wii: bool,
    gamecube: bool,
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
    pub fn shows_game(&self, game: &Game) -> bool {
        match game.console {
            ConsoleType::GameCube => self.gamecube,
            ConsoleType::Wii => self.wii,
        }
    }
}

/// Renders the console filter controls
pub fn ui_console_filter(ui: &mut egui::Ui, filter: &mut ConsoleFilter) {
    ui.horizontal(|ui| {
        for console_type in ConsoleType::iter() {
            ui.checkbox(
                &mut filter.wii,
                format!("{} Show {}", console_type.icon(), console_type),
            );
        }
    });
}
