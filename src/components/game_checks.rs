// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::Game;
use eframe::egui;

pub fn ui_game_checks(ui: &mut egui::Ui, game: &Game) {
    // Verified label
    match game.is_verified {
        Some(true) => {
            ui.label("✅").on_hover_text("✅ Game is Redump verified");
        }
        Some(false) => {
            ui.colored_label(egui::Color32::DARK_RED, "❌")
                .on_hover_text(
                    "❌ Game is not Redump verified, an 🔎 integrity check is recommended",
                );
        }
        None => {
            // We don't know if the game is verified, so we don't show anything
        }
    }

    // Corrupt label
    match game.is_corrupt {
        Some(true) => {
            ui.colored_label(egui::Color32::DARK_RED, "💔")
                .on_hover_text("💔 Game is corrupt");
        }
        Some(false) => {
            ui.label("💖").on_hover_text("💖 Game is intact");
        }
        None => {
            // We don't know if the game is corrupt, so we don't show anything
        }
    }
}
