// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, wiiload};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    if app.choose_file_to_push.show(ctx).selected()
        && let Some(path) = app.choose_file_to_push.path()
    {
        wiiload::spawn_push_file_task(
            path.to_path_buf(),
            app.config.contents.wii_ip.clone(),
            &app.task_processor,
        );
    }
}
