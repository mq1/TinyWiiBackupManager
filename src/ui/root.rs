// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    app::{AppState, UiBuffers},
    ui::{self},
};
use eframe::egui;

pub fn update(ctx: &egui::Context, app_state: &AppState, ui_buffers: &mut UiBuffers) {
    ui::nav::update(ctx, app_state, ui_buffers);

    match app_state.current_view {
        ui::View::Games => ui::games::update(ctx, app_state, ui_buffers),
        ui::View::HbcApps => ui::hbc_apps::update(ctx, app_state, ui_buffers),
        ui::View::Osc => ui::osc::update(ctx, app_state, ui_buffers),
        ui::View::Wiiload => ui::wiiload::update(ctx, app_state, ui_buffers),
        ui::View::Tools => ui::tools::update(ctx, app_state, ui_buffers),
        ui::View::Settings => ui::settings::update(ctx, app_state, ui_buffers),
    }

    ui::status::update(ctx, app_state, ui_buffers);

    if let Some(modal) = &app_state.current_modal {
        match modal {
            ui::Modal::DeleteGame(game_i) => {
                ui::delete_game::update(ctx, app_state, ui_buffers, *game_i)
            }
            ui::Modal::DeleteHbcApp(hbc_app_i) => {
                ui::delete_hbc_app::update(ctx, app_state, ui_buffers, *hbc_app_i)
            }
            ui::Modal::GameInfo(game_i, disc_info, game_info) => {
                ui::game_info::update(ctx, app_state, ui_buffers, *game_i, disc_info, game_info)
            }
            ui::Modal::HbcAppInfo(hbc_app_i) => {
                ui::hbc_app_info::update(ctx, app_state, ui_buffers, *hbc_app_i)
            }
            ui::Modal::ConvertGames(discs) => {
                ui::confirm_conversion::update(ctx, app_state, ui_buffers, discs)
            }
            ui::Modal::Info => ui::info::update(ctx, app_state, ui_buffers),
        }
    }
}
