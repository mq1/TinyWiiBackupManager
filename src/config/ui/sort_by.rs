// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy, message::ConfigMessage},
    ui::components::dual_toggle::{DualToggleItem, dual_toggle},
};
use iced::Element;
use lucide_icons::Icon;

fn sort_by_name(config: &Config) -> DualToggleItem<'static, ConfigMessage> {
    let current_sort_by = config.sort_by();

    let icon = match current_sort_by {
        SortBy::NameDescending => Icon::ArrowUpAZ,
        _ => Icon::ArrowDownAZ,
    };

    let on_press = match current_sort_by {
        SortBy::NameAscending => ConfigMessage::SetSortBy(SortBy::NameDescending),
        _ => ConfigMessage::SetSortBy(SortBy::NameAscending),
    };

    let active = matches!(
        current_sort_by,
        SortBy::NameAscending | SortBy::NameDescending
    );

    DualToggleItem {
        icon,
        desc: "Sort by name",
        active,
        on_press,
    }
}

fn sort_by_size(config: &Config) -> DualToggleItem<'static, ConfigMessage> {
    let current_sort_by = config.sort_by();

    let icon = match current_sort_by {
        SortBy::SizeDescending => Icon::ArrowUp01,
        _ => Icon::ArrowDown01,
    };

    let on_press = match current_sort_by {
        SortBy::SizeAscending => ConfigMessage::SetSortBy(SortBy::SizeDescending),
        _ => ConfigMessage::SetSortBy(SortBy::SizeAscending),
    };

    let active = matches!(
        current_sort_by,
        SortBy::SizeAscending | SortBy::SizeDescending
    );

    DualToggleItem {
        icon,
        desc: "Sort by size",
        active,
        on_press,
    }
}

pub fn sort_by(config: &Config) -> Element<'_, ConfigMessage> {
    dual_toggle([sort_by_name(config), sort_by_size(config)])
}
