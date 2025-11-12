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
        ui::View::Wiiload => ui::wiiload::update(ctx, app),
        ui::View::Tools => ui::tools::update(ctx, app),
        ui::View::Settings => ui::settings::update(ctx, app),
    }

    ui::status::update(ctx, app);

    match app.current_modal {
        ui::Modal::None => {}
        ui::Modal::DeleteGame(game_i) => ui::delete_game::update(ctx, app, game_i),
        ui::Modal::DeleteHbcApp(hbc_app_i) => ui::delete_hbc_app::update(ctx, app, hbc_app_i),
        ui::Modal::GameInfo => ui::game_info::update(ctx, app),
        ui::Modal::HbcAppInfo(hbc_app_i) => ui::hbc_app_info::update(ctx, app, hbc_app_i),
        ui::Modal::ConvertGames => ui::confirm_conversion::update(ctx, app),
        ui::Modal::Info => ui::info::update(ctx, app),
    }

    ui::choose_mount_point_dialog::update(ctx, app);
    ui::choose_games_dialog::update(ctx, app);
    ui::choose_hbc_apps_dialog::update(ctx, app);
    ui::choose_archive_path_dialog::update(ctx, app);
    ui::choose_file_to_push_dialog::update(ctx, app);

    app.notifications.show_toasts(ctx);
}
