// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message, osc::osc_state::OscAppList, ui::components::search_bar::search_bar,
};
use iced::Element;

pub fn osc_app_search(apps: &OscAppList) -> Element<'_, Message> {
    search_bar(
        &apps.filter.search_term,
        Message::SearchOscApps,
        "Search by Name",
    )
}
