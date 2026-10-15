// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::{Config, SortBy, message::ConfigMessage},
    ui::components::dual_toggle::{DualToggleItem, dual_toggle},
};
use iced::Element;
use lucide_iced::bytes::{ARROW_DOWN_0_1, ARROW_DOWN_A_Z, ARROW_UP_0_1, ARROW_UP_A_Z};

fn sort_by_name(config: &Config) -> DualToggleItem<'static, ConfigMessage> {
    let current_sort_by = config.sort_by();

    let icon_bytes = match current_sort_by {
        SortBy::NameDescending => ARROW_UP_A_Z,
        _ => ARROW_DOWN_A_Z,
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
        icon_bytes,
        desc: "Sort by name",
        active,
        on_press,
    }
}

fn sort_by_size(config: &Config) -> DualToggleItem<'static, ConfigMessage> {
    let current_sort_by = config.sort_by();

    let icon_bytes = match current_sort_by {
        SortBy::SizeDescending => ARROW_UP_0_1,
        _ => ARROW_DOWN_0_1,
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
        icon_bytes,
        desc: "Sort by size",
        active,
        on_press,
    }
}

pub fn sort_by(config: &Config) -> Element<'_, ConfigMessage> {
    dual_toggle([sort_by_name(config), sort_by_size(config)])
}
