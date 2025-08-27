use crate::app::App;
use crate::game::Game;
use crate::update_check::UpdateInfo;
use anyhow::Error;
use eframe::egui;
use log::error;

/// Messages that can be sent from background tasks to the main thread
#[derive(Debug)]
pub enum BackgroundMessage {
    /// Signal for current operation progress (kept for compatibility with convert/verify functions)
    OperationProgress(u64, u64),
    /// Signal that the directory has changed
    DirectoryChanged,
    /// Signal that an error occurred
    Error(Error),
    /// Signal that an update is available
    UpdateCheckComplete(Option<UpdateInfo>),
    /// Signal to start verification of a single game
    StartSingleVerification(Box<Game>),
}

/// Processes messages received from background tasks
pub fn handle_messages(app: &mut App, ctx: &egui::Context) {
    let sender = app.inbox.sender();

    for msg in app.inbox.read(ctx) {
        match msg {
            BackgroundMessage::OperationProgress(_progress, _total) => {
                // This message is kept for compatibility but no longer processed here
                // The job system handles progress directly
            }

            BackgroundMessage::DirectoryChanged => {
                if let Err(e) = app.refresh_games() {
                    let _ = sender.send(BackgroundMessage::Error(e));
                }
            }

            BackgroundMessage::Error(e) => {
                error!("{e:?}");
                app.bottom_right_toasts.error(e.to_string());
            }

            BackgroundMessage::UpdateCheckComplete(update_info) => {
                if let Some(update_info) = update_info {
                    let update_text = format!("✨Update available: {}✨    ", update_info.version);

                    app.bottom_left_toasts.custom(
                        update_text,
                        "⬇".to_string(),
                        egui::Color32::GRAY,
                    );

                    app.update_info = Some(update_info);
                }
            }

            BackgroundMessage::StartSingleVerification(game) => {
                // Start verification of a single game
                app.spawn_verification(vec![game]);
            }
        }
    }
}
