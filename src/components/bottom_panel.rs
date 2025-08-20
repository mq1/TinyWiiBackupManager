// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::{app::App, components::update_notifier::ui_update_notifier};
use eframe::egui;

// --- UI Rendering ---

/// Renders the bottom panel, which includes the update notifier and other controls.
pub fn ui_bottom_panel(ctx: &egui::Context, app: &mut App) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui_update_notifier(ui, app);

            // Layout for other controls, aligned to the right.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut app.remove_sources, "💣 Remove sources")
                    .on_hover_text("⚠ DANGER ⚠\n\nThis will delete the input files!");
            });
        });
    });
}
