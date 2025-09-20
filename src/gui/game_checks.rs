// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::game::Game;
use eframe::egui;

pub fn ui_game_checks(ui: &mut egui::Ui, game: &Game) {
    // Verified label
    match game.is_verified {
        Some(true) => {
            ui.label(egui_phosphor::regular::CHECK)
                .on_hover_text(format!(
                    "{} Game is Redump verified",
                    egui_phosphor::regular::CHECK
                ));
        }
        Some(false) => {
            ui.colored_label(egui::Color32::DARK_RED, egui_phosphor::regular::CROSS)
                .on_hover_text(format!(
                    "{} Game is not Redump verified, an {} integrity check is recommended",
                    egui_phosphor::regular::CROSS,
                    egui_phosphor::regular::MAGNIFYING_GLASS
                ));
        }
        None => {
            // We don't know if the game is verified, so we don't show anything
        }
    }

    // Corrupt label
    match game.is_corrupt {
        Some(true) => {
            ui.colored_label(egui::Color32::DARK_RED, egui_phosphor::regular::SEAL_QUESTION)
                .on_hover_text(format!("{} Hashes don't match: the game has been altered\n\nIt can indicate that a partition has been removed or a potential data corruption", egui_phosphor::regular::SEAL_QUESTION));
        }
        Some(false) => {
            ui.label(egui_phosphor::regular::SEAL_CHECK)
                .on_hover_text(format!(
                    "{} Game is intact",
                    egui_phosphor::regular::SEAL_CHECK
                ));
        }
        None => {
            // We don't know if the game is corrupt, so we don't show anything
        }
    }
}
