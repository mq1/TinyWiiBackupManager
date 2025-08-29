// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::components;
use crate::update_check::UpdateInfo;
use anyhow::Error;
use eframe::egui;
use log::{error, info};

/// Messages that can be sent from background tasks to the main thread
#[derive(Debug, Clone)]
pub enum BackgroundMessage {
    /// Signal that an error occurred
    Error(String),
    /// Informational message
    Info(String),
    /// Signal that the base directory has changed
    DirectoryChanged,
    /// Signal that update checking has completed
    GotUpdate(Option<UpdateInfo>),
}

/// Implement the From trait to automatically convert anyhow::Error into our message.
impl From<Error> for BackgroundMessage {
    fn from(e: Error) -> Self {
        BackgroundMessage::Error(format!("{e}"))
    }
}

/// Processes messages received from background tasks
pub fn handle_messages(app: &mut App, ctx: &egui::Context) {
    let sender = app.inbox.sender();

    for msg in app.inbox.read(ctx) {
        match msg {
            BackgroundMessage::DirectoryChanged => {
                if let Err(e) = app.refresh_games() {
                    let _ = sender.send(e.into());
                }
            }

            BackgroundMessage::Error(e) => {
                error!("{e}");
                components::toasts::show_error_toast(app, &e);
            }

            BackgroundMessage::Info(e) => {
                info!("{e}");
                components::toasts::show_info_toast(app, &e);
            }

            BackgroundMessage::GotUpdate(update) => {
                components::toasts::show_update_toast(app, &update);
                app.update_info = update;
            }
        }
    }
}
