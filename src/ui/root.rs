// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::ui::{self};
use eframe::egui;

pub fn update(ctx: &egui::Context, frame: &eframe::Frame, app: &mut App) {
    ui::nav::update(ctx, frame, app);

    match app.current_view {
        ui::View::Games => ui::games::update(ctx, frame, app),
        ui::View::HbcApps => ui::hbc_apps::update(ctx, frame, app),
        ui::View::Osc => ui::osc::update(ctx, app),
        ui::View::Wiiload => ui::wiiload::update(ctx, frame, app),
        ui::View::Tools => ui::tools::update(ctx, frame, app),
        ui::View::Settings => ui::settings::update(ctx, app),
    }

    ui::status::update(ctx, app);

    if let Some(modal) = &app.current_modal {
        match modal {
            ui::Modal::GameInfo(game_i, disc_info, game_info) => {
                ui::game_info::update(ctx, app, *game_i, disc_info, game_info)
            }
            ui::Modal::HbcAppInfo(hbc_app_i) => ui::hbc_app_info::update(ctx, app, *hbc_app_i),
            ui::Modal::OscAppInfo(osc_app_i) => ui::osc_app_info::update(ctx, app, *osc_app_i),
            ui::Modal::Info => ui::info::update(ctx, app),
        }
    }

    app.notifications.show_toasts(ctx);
}
