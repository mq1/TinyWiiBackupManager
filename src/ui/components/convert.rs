// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    messages::Message,
    ui::components::{my_button::my_button, my_group::my_group, my_link::my_link},
};
use iced::{Alignment, Element, widget::row};
use lucide_icons::Icon;

const NOD_URL: &str = "https://github.com/encounter/nod";

pub fn convert() -> Element<'static, Message> {
    my_group(
        "Manual conversion (using nod)",
        row![
            my_button()
                .icon(Icon::Play)
                .on_press(Message::PickGameToConvert),
            "Select in-path and out-path and run the conversion"
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .icon(Icon::Shuffle)
    .top_right_content(my_link(NOD_URL, NOD_URL))
    .into()
}
