// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_button::MyButton};
use iced::{
    Element,
    widget::{row, space, text},
};
use lucide_icons::{Icon, iced::icon_chevron_right};

pub fn view<'a>() -> Element<'a, Message> {
    row![
        icon_chevron_right().size(20),
        text("Homebrew Apps").size(20),
        space::horizontal(),
        MyButton::new().icon(Icon::RotateCw).rounded().view()
    ]
    .spacing(5)
    .into()
}
