// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, ui};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    ui::nav::update(ctx, app);

    match app.current_view {
        ui::View::Games => ui::games::update(ctx, app),
        ui::View::HbcApps => ui::hbc_apps::update(ctx, app),
        ui::View::Osc => ui::osc::view(ctx),
        ui::View::Settings => ui::settings::view(ctx),
    }

    ui::status::update(ctx, app);
    ui::remove_game::update(ctx, app);
    ui::disc_info::update(ctx, app);
    ui::choose_mount_point_dialog::view(ctx, app);

    app.toasts.lock().show(ctx);
}
