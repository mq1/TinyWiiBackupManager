// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::components;
use crate::update_check::UpdateInfo;
use anyhow::Error;
use eframe::egui;
use tracing::{error, info};
use std::sync::Arc;

/// Messages that can be sent from background tasks to the main thread
#[derive(Debug, Clone)]
pub enum BackgroundMessage {
    /// Signal that an error occurred
    Error(Arc<Error>),
    /// Informational message
    Info(String),
    /// Signal that the base directory has changed
    DirectoryChanged,
    /// Signal that update checking has completed
    GotUpdate(Option<UpdateInfo>),
    /// Signal that a new cover has been downloaded
    NewCover(String),
    /// Signal that the status has changed
    UpdateStatus(Option<String>),
}

/// Implement the From trait to automatically convert anyhow::Error into our message.
impl From<Error> for BackgroundMessage {
    fn from(e: Error) -> Self {
        BackgroundMessage::Error(Arc::new(e))
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
                error!("{e:?}");
                let msg = format!("{e:#}");
                components::toasts::show_error_toast(app, &msg);
            }

            BackgroundMessage::Info(msg) => {
                info!("{msg}");
                components::toasts::show_info_toast(app, &msg);
            }

            BackgroundMessage::GotUpdate(update) => {
                components::toasts::show_update_toast(app, &update);
                app.update_info = update;
            }

            BackgroundMessage::NewCover(id) => {
                let msg = format!("Downloaded cover for {}", id);
                let _ = sender.send(BackgroundMessage::Info(msg));

                // we need to refresh the image cache
                // we can forget only the image that changed with ctx.forget_image(uri) but this works fine
                ctx.forget_all_images();
            }

            BackgroundMessage::UpdateStatus(status) => {
                app.task_status = status;
            }
        }
    }
}
