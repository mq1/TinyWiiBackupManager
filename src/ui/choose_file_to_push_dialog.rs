// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, wiiload};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    app.choose_file_to_push.update(ctx);

    if let Some(path) = app.choose_file_to_push.take_picked() {
        wiiload::spawn_push_file_task(
            path,
            app.config.contents.wii_ip.clone(),
            &app.task_processor,
        );
    }
}
