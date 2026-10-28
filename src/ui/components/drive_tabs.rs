// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::AppState,
    ui::{
        components::my_button::{MyButtonKind, my_button},
        pages::Page,
    },
};
use iced::{
    Alignment, Element,
    widget::{row, space, text},
};
use lucide_icons::Icon;

/// Above a drive's games and homebrew: back to My Consoles, the drive and its tabs
pub fn drive_tabs(state: &AppState) -> Element<'_, Message> {
    let mount_point = state.config.mount_point();
    let label = mount_point
        .file_name()
        .unwrap_or(mount_point.as_os_str())
        .to_string_lossy()
        .into_owned();

    let tab = |label: &'static str, icon: Icon, page: Page| {
        my_button::<Message>()
            .icon(icon)
            .label(label)
            .kind(if state.current_page == page {
                MyButtonKind::Primary
            } else {
                MyButtonKind::Secondary
            })
            .on_press(Message::NavigateTo(page))
    };

    row![
        my_button::<Message>()
            .icon(Icon::ArrowLeft)
            .kind(MyButtonKind::Toolbar)
            .on_press(Message::NavigateTo(Page::Consoles)),
        text(label).size(18),
        space::horizontal(),
        tab("Games", Icon::Gamepad2, Page::Games),
        tab("Homebrew", Icon::Bubbles, Page::HomebrewApps),
    ]
    .spacing(10)
    .padding([10, 10])
    .align_y(Alignment::Center)
    .into()
}
