use crate::app::{App, OperationState, OperationType};
use crate::messages::BackgroundMessage;
use eframe::egui;
use size::Size;

pub fn ui_operation_modal(ctx: &egui::Context, app: &App) {
    if let OperationState::InProgress {
        operation,
        total_items,
        items_completed,
        current_item,
        current_progress,
        ..
    } = &app.operation_state
    {
        let modal_id = match operation {
            OperationType::Converting => "conversion_modal",
            OperationType::Verifying => "verification_modal",
        };

        egui::Modal::new(egui::Id::new(modal_id)).show(ctx, |ui| {
            ui.set_min_width(400.0);
            ui.vertical_centered(|ui| {
                // Header
                let heading = match operation {
                    OperationType::Converting => "📦 Converting Files",
                    OperationType::Verifying => "🔍 Verifying Disc Integrity",
                };
                ui.heading(heading);
                ui.separator();
                ui.add_space(10.0);

                // Show overall progress if processing multiple items
                if *total_items > 1 {
                    let item_label = match operation {
                        OperationType::Converting => "File",
                        OperationType::Verifying => "Game",
                    };
                    ui.label(format!(
                        "{} {} of {}",
                        item_label,
                        items_completed + 1,
                        total_items
                    ));
                    ui.add_space(5.0);
                }

                // Show current item name
                if !current_item.is_empty() {
                    ui.label(egui::RichText::new(current_item).strong());
                    ui.add_space(5.0);
                }

                ui.spinner();
                ui.add_space(10.0);

                let (current, total) = current_progress;
                if *total > 0 {
                    // Calculate combined progress
                    let base_progress = *items_completed as f32 / *total_items as f32;
                    let current_item_progress =
                        (*current as f32 / *total as f32) / *total_items as f32;
                    let progress_ratio = base_progress + current_item_progress;

                    ui.add(egui::ProgressBar::new(progress_ratio).show_percentage());
                    ui.add_space(5.0);

                    ui.label(format!(
                        "Progress: {} / {}",
                        Size::from_bytes(*current),
                        Size::from_bytes(*total)
                    ));
                } else {
                    let initializing_msg = match operation {
                        OperationType::Converting => "Initializing conversion...",
                        OperationType::Verifying => "Initializing verification...",
                    };
                    ui.label(initializing_msg);
                }

                ui.add_space(10.0);

                // Cancel button
                if ui.button("Cancel").clicked() {
                    let _ = app.inbox.sender().send(BackgroundMessage::CancelOperation);
                }
            });
        });
    }
}
