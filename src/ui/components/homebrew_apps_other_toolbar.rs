// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message, state::AppState, ui::components::homebrew_app_search::homebrew_app_search,
};
use iced::{Element, widget::row};

pub fn homebrew_apps_other_toolbar(state: &AppState) -> Element<'_, Message> {
    row![homebrew_app_search(state)].into()
}
