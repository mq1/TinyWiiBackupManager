// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{APP_ICON, message::Message, state::State, ui::style};
use iced::{
    Alignment, Element, padding,
    widget::{button, column, container, image, row, rule, space, text},
};
use lucide_icons::iced::{
    icon_folder, icon_github, icon_globe, icon_heart, icon_scroll_text, icon_triangle,
    icon_triangle_alert, icon_wand_sparkles,
};

const VERSION_TEXT: &str = concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION"));
const COPYRIGHT_TEXT: &str = "Copyright © 2026 Manuel Quarneti";
const REPO_URI: &str = env!("CARGO_PKG_REPOSITORY");
const WIKI_URI: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/wiki");
const LICENSE_URI: &str = "https://www.gnu.org/licenses/gpl-3.0.html";

pub fn view(state: &State) -> Element<'_, Message> {
    let icon_handle = image::Handle::from_rgba(256, 256, &APP_ICON[..]);

    column![
        space::vertical(),
        container(
            container(
                column![
                    row![
                        image(icon_handle).width(100).height(100),
                        column![
                            text(VERSION_TEXT).size(20),
                            text(COPYRIGHT_TEXT),
                            button(
                                row![
                                    icon_scroll_text().style(text::primary),
                                    text("GPL-3.0-only").style(text::primary)
                                ]
                                .spacing(5)
                            )
                            .style(button::text)
                            .padding(padding::top(2))
                            .on_press_with(|| Message::OpenThat(LICENSE_URI.into())),
                        ],
                    ].spacing(10).padding(padding::bottom(10)).align_y(Alignment::Center),
                    row![
                        icon_triangle_alert(),
                        text("TinyWiiBackupManager is intended strictly for legal homebrew use and is not affiliated with or endorsed by Nintendo. Use of TinyWiiBackupManager for pirated or unauthorized copies of games is strictly prohibited.")
                    ].spacing(5),
                    space(),
                    rule::horizontal(1),
                    space(),
                    row![icon_heart().size(18), text("Special thanks to").size(18)].spacing(5),
                    row![button(row![icon_triangle().style(text::primary), text("Luke Street").style(text::primary)].spacing(5)).style(button::text).padding(0).on_press_with(|| Message::OpenThat("https://github.com/encounter".into())), "for developing nod and helping TWBM leverage it effectively."].spacing(5),
                    row![button(row![icon_wand_sparkles().style(text::primary), text("blackb0x").style(text::primary)].spacing(5)).style(button::text).padding(0).on_press_with(|| Message::OpenThat("https://github.com/wiidev".into())), "for invaluable feedback and advice during TWBM's development."].spacing(5)
                ]
                .spacing(5)
            )
            .padding(40)
            .style(style::card)
        ).padding(40),
        space::vertical(),
        row![
            button(row![icon_folder(), text("Data Directory")].spacing(5))
                .style(style::rounded_button)
                .on_press_with(|| Message::OpenThat(state.data_dir.as_os_str().into())),
            button(row![icon_github(), text("Source Code")].spacing(5))
                .style(style::rounded_button)
                .on_press_with(|| Message::OpenThat(REPO_URI.into())),
            button(row![icon_globe(), text("Wiki")].spacing(5))
                .style(style::rounded_button)
                .on_press_with(|| Message::OpenThat(WIKI_URI.into())),
            space::horizontal(),
        ]
        .padding(10)
        .spacing(5)
    ]
    .align_x(Alignment::Center)
    .into()
}
