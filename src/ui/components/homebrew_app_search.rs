// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_state::HomebrewAppList, messages::Message,
    ui::components::search_bar::search_bar,
};
use iced::Element;

pub fn homebrew_app_search(apps: &HomebrewAppList) -> Element<'_, Message> {
    search_bar(
        apps.search_term(),
        Message::SearchHomebrewApps,
        "Search by Name",
    )
}
