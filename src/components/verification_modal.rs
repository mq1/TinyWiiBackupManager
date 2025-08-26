use crate::app::{App, VerificationState};
use crate::messages::BackgroundMessage;
use eframe::egui;
use size::Size;

pub fn ui_verification_modal(ctx: &egui::Context, app: &App) {
    if let VerificationState::Verifying {
        ref game_path,
        progress,
        total_games,
        games_verified,
        ..
    } = app.verification_state
    {
        egui::Modal::new(egui::Id::new("verification_modal")).show(ctx, |ui| {
            ui.set_min_width(400.0);
            ui.vertical_centered(|ui| {
                ui.heading("🔍 Verifying Disc Integrity");
                ui.separator();
                ui.add_space(10.0);

                // Show overall progress if verifying multiple games
                if total_games > 1 {
                    ui.label(format!("Game {} of {}", games_verified + 1, total_games));
                    ui.add_space(5.0);
                }

                // Find the game by path to display its title
                if let Some(game) = app.games.iter().find(|g| &g.path == game_path) {
                    ui.label(&game.display_title);
                    ui.add_space(10.0);
                }

                ui.spinner();
                ui.add_space(10.0);

                let (current_progress, total) = progress;
                if total > 0 {
                    let progress_ratio = current_progress as f32 / total as f32;
                    ui.add(egui::ProgressBar::new(progress_ratio).show_percentage());
                    ui.add_space(5.0);

                    ui.label(format!(
                        "Progress: {} / {}",
                        Size::from_bytes(current_progress),
                        Size::from_bytes(total)
                    ));
                } else {
                    ui.label("Initializing verification...");
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
