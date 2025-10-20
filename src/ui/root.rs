// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{app::App, ui};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    ui::nav::update(ctx, app);

    match app.current_view {
        ui::View::Games => ui::games::update(ctx, app),
        ui::View::HbcApps => ui::hbc_apps::update(ctx, app),
        ui::View::Osc => ui::osc::update(ctx, app),
        ui::View::Settings => ui::settings::update(ctx, app),
    }

    ui::status::update(ctx, app);
    ui::remove_game::update(ctx, app);
    ui::disc_info::update(ctx, app);
    ui::hbc_app_info::update(ctx, app);
    ui::choose_mount_point_dialog::update(ctx, app);
    ui::choose_games_dialog::update(ctx, app);
    ui::choose_hbc_apps_dialog::update(ctx, app);

    app.toasts.show(ctx);
}
