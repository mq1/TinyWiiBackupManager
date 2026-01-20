// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State, ui::style};
use iced::{
    Element, Length,
    widget::{column, container, radio, row, rule, space, text},
};
use lucide_icons::iced::{icon_disc_3, icon_settings};

pub fn view(state: &State) -> Element<'_, Message> {
    let wii_output_format = state.config.wii_output_format();

    column![
        row![icon_settings().size(18), text("Settings").size(18)].spacing(5),
        container(
            column![
                row![icon_disc_3(), text("Wii Output Format")].spacing(5),
                rule::horizontal(1),
                space(),
                radio(
                    "WBFS (recommended)",
                    nod::common::Format::Wbfs,
                    Some(wii_output_format),
                    |format| Message::UpdateConfig(
                        state.config.clone().with_wii_output_format(format)
                    )
                ),
                radio(
                    "ISO (very large)",
                    nod::common::Format::Iso,
                    Some(wii_output_format),
                    |format| Message::UpdateConfig(
                        state.config.clone().with_wii_output_format(format)
                    )
                ),
            ]
            .spacing(5)
            .padding(10)
            .width(Length::Fill)
        )
        .style(style::card)
    ]
    .spacing(10)
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
