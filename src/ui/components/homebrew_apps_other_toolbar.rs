// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_state::HomebrewAppList, messages::Message,
    ui::components::homebrew_app_search::homebrew_app_search, util::drive_state::DriveState,
};
use iced::{
    Alignment, Element,
    widget::{Text, row, space, text},
};

fn usage<'a>(apps: &'a HomebrewAppList, drive: &'a DriveState) -> Text<'a> {
    text!(
        "{} app{}   ~ {}  ",
        apps.count(),
        if apps.count() == 1 { "" } else { "s" },
        if let DriveState::Loaded(info) = drive {
            info.apps_size_str()
        } else {
            ""
        }
    )
}

pub fn homebrew_apps_other_toolbar<'a>(
    apps: &'a HomebrewAppList,
    drive: &'a DriveState,
) -> Element<'a, Message> {
    row![
        homebrew_app_search(apps),
        space::horizontal(),
        usage(apps, drive)
    ]
    .align_y(Alignment::Center)
    .into()
}
