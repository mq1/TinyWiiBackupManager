// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use crate::app::App;
use crate::game::Game;
use crate::gui;
use crate::util::oscwii;
use crate::util::update_check::UpdateInfo;
use anyhow::Error;
use eframe::egui;
use std::sync::Arc;
use tracing::{error, info};

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
    NewCover(Game),
    /// Signal that the status has changed
    UpdateStatus(String),
    /// Signal that the status should be cleared
    ClearStatus,
    /// Signal that covers should be downloaded
    TriggerDownloadCovers,
    /// Signal that the OSCWii app list has to be updated
    GotNewAppCache(oscwii::AppCache),
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
                if let Err(e) = app.refresh_wiiapps() {
                    let _ = sender.send(e.into());
                }
                if let Err(e) = app.refresh_fs() {
                    let _ = sender.send(e.into());
                }
            }

            BackgroundMessage::Error(e) => {
                error!("{e:?}");
                let msg = format!("{e:#}");
                gui::toasts::show_error_toast(app, &msg);
            }

            BackgroundMessage::Info(msg) => {
                info!("{msg}");
                gui::toasts::show_info_toast(app, &msg);
            }

            BackgroundMessage::GotUpdate(update) => {
                gui::toasts::show_update_toast(app, &update);
                app.update_info = update;
            }

            BackgroundMessage::NewCover(game) => {
                if let Some(base_dir) = &app.base_dir {
                    let uri = game.get_local_cover_uri(base_dir.cover_dir());
                    ctx.forget_image(&uri);
                }
            }

            BackgroundMessage::UpdateStatus(status) => {
                app.task_status = Some(status);
            }

            BackgroundMessage::ClearStatus => {
                app.task_status.take();
            }

            BackgroundMessage::TriggerDownloadCovers => {
                app.download_covers();
            }

            BackgroundMessage::GotNewAppCache(cache) => {
                app.oscwii_apps = cache;

                if let Err(e) = app.refresh_wiiapps() {
                    let _ = sender.send(e.into());
                }
            }
        }
    }
}
