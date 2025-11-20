// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::app::App;
use crate::ui::{self};
use eframe::egui;

pub fn update(ctx: &egui::Context, app: &mut App) {
    ui::nav::update(ctx, app);

    match app.current_view {
        ui::View::Games => ui::games::update(ctx, app),
        ui::View::HbcApps => ui::hbc_apps::update(ctx, app),
        ui::View::Osc => ui::osc::update(ctx, app),
        ui::View::Wiiload => ui::wiiload::update(ctx, app),
        ui::View::Tools => ui::tools::update(ctx, app),
        ui::View::Settings => ui::settings::update(ctx, app),
    }

    ui::status::update(ctx, app);

    if let Some(modal) = &app.current_modal {
        match modal {
            ui::Modal::DeleteGame(game_i) => ui::delete_game::update(ctx, app, *game_i),
            ui::Modal::DeleteHbcApp(hbc_app_i) => ui::delete_hbc_app::update(ctx, app, *hbc_app_i),
            ui::Modal::GameInfo(game_i, disc_info, game_info) => {
                ui::game_info::update(ctx, app, *game_i, disc_info, game_info)
            }
            ui::Modal::HbcAppInfo(hbc_app_i) => ui::hbc_app_info::update(ctx, app, *hbc_app_i),
            ui::Modal::OscAppInfo(osc_app_i) => ui::osc_app_info::update(ctx, app, *osc_app_i),
            ui::Modal::ConvertGames(discs) => ui::confirm_conversion::update(ctx, app, discs),
            ui::Modal::Info => ui::info::update(ctx, app),
        }
    }

    app.choose_games.update(ctx);
    app.choose_hbc_apps.update(ctx);
    app.choose_mount_point.update(ctx);
    app.choose_archive_path.update(ctx);
    app.choose_file_to_push.update(ctx);
    app.notifications.show_toasts(ctx);
}
