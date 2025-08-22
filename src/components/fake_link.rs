// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use eframe::egui::{self, RichText};

/// A label that behaves like a link.
/// It only shows an underline when hovered.
pub fn fake_link(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let response = ui
        .add(egui::Label::new(
            RichText::new(text).color(ui.visuals().hyperlink_color),
        ))
        .on_hover_cursor(egui::CursorIcon::PointingHand);

    // Add underline on hover
    if response.hovered() {
        let rect = response.rect;
        ui.painter().hline(
            rect.x_range(),
            rect.bottom(),
            egui::Stroke::new(1.5, ui.visuals().hyperlink_color),
        );
    }

    response
}
