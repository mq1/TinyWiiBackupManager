// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Element,
    widget::{button, column, image, row, rule, space, text},
};
use iced_fonts::lucide;

pub fn view(state: &State, osc_i: usize) -> Element<'_, Message> {
    let app = &state.osc_apps[osc_i];

    let mut row = row![column![
        row![
            lucide::shopping_bag().size(18),
            text(&app.meta.name).size(18)
        ]
        .spacing(5),
        rule::horizontal(1),
        space::vertical(),
        row![
            button(row![lucide::globe(), text("Open OSC Page")].spacing(5))
                .style(style::rounded_button)
                .on_press(Message::OpenOscPage(osc_i)),
            button(row![lucide::monitor_up(), text("Send via Wiiload")].spacing(5))
                .style(style::rounded_button),
            button(row![lucide::cloud_download(), text("Download")].spacing(5))
                .style(style::rounded_button)
        ]
        .spacing(5)
    ]]
    .padding(10);

    if let Some(icon) = state.get_osc_app_icon(app) {
        row = row.push(
            button(image(icon).height(50))
                .style(button::text)
                .on_press(Message::OpenOscIcon(osc_i)),
        );
    }

    row.into()
}
