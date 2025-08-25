use crate::app::{App, ConversionState};
use crate::update_check::UpdateInfo;
use anyhow::Error;
use eframe::egui;
use log::error;

/// Messages that can be sent from background tasks to the main thread
#[derive(Debug)]
pub enum BackgroundMessage {
    /// Signal for current file conversion progress
    ConversionProgress(u64, u64),
    /// Signal that a single file conversion has completed
    FileConverted,
    /// Signal that the conversion has completed (with result)
    ConversionComplete,
    /// Signal that the directory has changed
    DirectoryChanged,
    /// Signal that an error occurred
    Error(Error),
    /// Signal that an update is available
    UpdateCheckComplete(Option<UpdateInfo>),
}

/// Processes messages received from background tasks
pub fn handle_messages(app: &mut App, ctx: &egui::Context) {
    let sender = app.inbox.sender();

    for msg in app.inbox.read(ctx) {
        match msg {
            BackgroundMessage::ConversionProgress(progress, total) => {
                if let ConversionState::Converting {
                    total_files,
                    files_converted,
                    ..
                } = app.conversion_state
                {
                    app.conversion_state = ConversionState::Converting {
                        total_files,
                        files_converted,
                        current_progress: (progress, total),
                    };
                }
            }

            BackgroundMessage::FileConverted => {
                if let ConversionState::Converting {
                    total_files,
                    files_converted,
                    ..
                } = app.conversion_state
                {
                    app.conversion_state = ConversionState::Converting {
                        total_files,
                        files_converted: files_converted + 1,
                        current_progress: (0, 0),
                    };
                }
            }

            BackgroundMessage::ConversionComplete => {
                app.conversion_state = ConversionState::Idle;
            }

            BackgroundMessage::DirectoryChanged => {
                if let Err(e) = app.refresh_games() {
                    let _ = sender.send(BackgroundMessage::Error(e));
                }
            }

            BackgroundMessage::Error(e) => {
                error!("{e:?}");
                let text = egui::RichText::new(e.to_string()).strong().size(16.0);
                app.bottom_right_toasts
                    .error(text)
                    .closable(true)
                    .duration(None);
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
        }
    }
}
