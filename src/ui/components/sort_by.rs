// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::SortBy,
    messages::Message,
    state::AppState,
    ui::components::dual_toggle::{DualToggleItem, dual_toggle},
};
use iced::Element;
use lucide_icons::Icon;

fn sort_by_name(state: &AppState) -> DualToggleItem<'static> {
    let icon = match state.config.sort_by {
        SortBy::NameDescending => Icon::ArrowUpAZ,
        _ => Icon::ArrowDownAZ,
    };

    let on_press = match state.config.sort_by {
        SortBy::NameAscending => Message::SetSortBy(SortBy::NameDescending),
        _ => Message::SetSortBy(SortBy::NameAscending),
    };

    let active = matches!(
        state.config.sort_by,
        SortBy::NameAscending | SortBy::NameDescending
    );

    DualToggleItem {
        icon,
        desc: "Sort by name",
        active,
        on_press,
    }
}

fn sort_by_size(state: &AppState) -> DualToggleItem<'static> {
    let icon = match state.config.sort_by {
        SortBy::SizeDescending => Icon::ArrowUp01,
        _ => Icon::ArrowDown01,
    };

    let on_press = match state.config.sort_by {
        SortBy::SizeAscending => Message::SetSortBy(SortBy::SizeDescending),
        _ => Message::SetSortBy(SortBy::SizeAscending),
    };

    let active = matches!(
        state.config.sort_by,
        SortBy::SizeAscending | SortBy::SizeDescending
    );

    DualToggleItem {
        icon,
        desc: "Sort by size",
        active,
        on_press,
    }
}

pub fn sort_by(state: &AppState) -> Element<'_, Message> {
    dual_toggle([sort_by_name(state), sort_by_size(state)])
}
