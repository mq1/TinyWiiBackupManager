// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    osc::osc_app::OscApp,
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
use lucide_icons::iced::{icon_calendar, icon_notepad_text, icon_tag};

pub fn osc_app_info(app: &OscApp) -> Element<'_, Message> {
    let content = row![
        column![
            row![icon_tag(), text!("Version: {}", app.version())].spacing(5),
            row![
                icon_calendar(),
                text!("Release date: {}", app.release_date_str())
            ]
            .spacing(5),
            row![get_dev_icon(app.coder()), text!("Coder: {}", app.coder())].spacing(5),
            row![
                icon_notepad_text(),
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
                my_link(app.osc_url(), app.osc_url())
            ]
            .spacing(10)
            .padding(20),
            space::vertical(),
            content,
            space::vertical(),
            rule::horizontal(1),
            row![
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
