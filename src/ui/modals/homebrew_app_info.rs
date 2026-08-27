// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_app::HomebrewApp,
    messages::Message,
    ui::{
        components::{
            my_button::{MyButtonKind, my_button},
            my_card::my_card,
            my_link::my_link,
        },
        developers::get_dev_icon,
    },
};
use iced::{
    Alignment, Element, Length,
    widget::{column, image, row, rule, space, text},
};
use lucide_icons::{
    Icon,
    iced::{icon_calendar, icon_notepad_text, icon_tag},
};

pub fn homebrew_app_info(app: &HomebrewApp) -> Element<'_, Message> {
    let content = row![
        column![
            row![icon_tag(), text!("Version: {}", &app.meta.version)].spacing(5),
            row![
                icon_calendar(),
                text!("Release date: {}", &app.meta.release_date)
            ]
            .spacing(5),
            row![
                get_dev_icon(&app.meta.coder),
                text!("Coder: {}", &app.meta.coder)
            ]
            .spacing(5),
            row![
                icon_notepad_text(),
                text!("Description: {}", &app.meta.short_description)
            ]
            .spacing(5),
            rule::horizontal(1),
            text(&app.meta.long_description)
                .height(100)
                .width(Length::Fill)
        ]
        .spacing(5),
        image(&app.icon).height(200),
    ]
    .padding(20)
    .spacing(50)
    .align_y(Alignment::Center);

    my_card(
        column![
            column![
                text(&app.meta.name).size(18),
                my_link(app.path.to_string_lossy(), || &app.path, Icon::Folder)
            ]
            .spacing(10)
            .padding(20),
            space::vertical(),
            content,
            space::vertical(),
            rule::horizontal(1),
            row![
                space(),
                my_link("Open Shop Channel page", || app.osc_url(), None),
                space::horizontal(),
                my_button("Close", None, MyButtonKind::Primary).on_press(Message::CloseModal)
            ]
            .align_y(Alignment::Center)
            .spacing(10)
            .padding(10)
        ]
        .width(600)
        .height(400),
    )
    .padding(0)
    .into()
}
