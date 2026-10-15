// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    homebrew::homebrew_app::HomebrewApp,
    message::Message,
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
use lucide_iced::{
    bytes::{CALENDAR, FOLDER, NOTEPAD_TEXT, TAG},
    themed_icon,
};

pub fn homebrew_app_info(app: &HomebrewApp) -> Element<'_, Message> {
    let content = row![
        column![
            row![themed_icon(TAG, 14.), text!("Version: {}", app.version())].spacing(5),
            row![
                themed_icon(CALENDAR, 14.),
                text!("Release date: {}", app.release_date())
            ]
            .spacing(5),
            row![get_dev_icon(app.coder()), text!("Coder: {}", app.coder())].spacing(5),
            row![
                themed_icon(NOTEPAD_TEXT, 14.),
                text!("Description: {}", app.short_description())
            ]
            .spacing(5),
            rule::horizontal(1),
            text(app.long_description()).height(100).width(Length::Fill)
        ]
        .spacing(5),
        app.icon().map(|icon| image(icon).height(200)),
    ]
    .padding(20)
    .spacing(50)
    .align_y(Alignment::Center);

    my_card(
        column![
            column![
                text(app.name()).size(18),
                my_link(app.path().to_string_lossy(), app.path()).icon(FOLDER)
            ]
            .spacing(10)
            .padding(20),
            space::vertical(),
            content,
            space::vertical(),
            rule::horizontal(1),
            row![
                space(),
                my_link("Open Shop Channel page", app.osc_url()),
                space::horizontal(),
                my_button()
                    .label("Close")
                    .kind(MyButtonKind::Primary)
                    .on_press(Message::CloseModal)
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
