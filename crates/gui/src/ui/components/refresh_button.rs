// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::AppState,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
    },
};
use iced::{Element, widget::tooltip};
use lucide_icons::Icon;
use twbm_driveinfo::DriveInfoState;
use twbm_games::manager::GameManager;
use twbm_homebrew::homebrew_app_manager::HomebrewAppManager;

pub fn refresh_button(state: &AppState) -> Element<'_, Message> {
    let mut refresh_btn = my_button().icon(Icon::RotateCw).kind(MyButtonKind::Toolbar);

    if !matches!(state.game_manager, GameManager::Loading)
        && !matches!(state.homebrew, HomebrewAppManager::Loading)
        && !matches!(state.drive, DriveInfoState::Loading)
    {
        refresh_btn = refresh_btn.on_press(Message::RefreshGamesAndApps);
    }

    tooltip(
        refresh_btn,
        my_card("Refresh games and apps"),
        tooltip::Position::Bottom,
    )
    .into()
}
