// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::style};
use bon::Builder;
use iced::{
    Element, Length,
    widget::{Text, button, column, container, row, rule, space, text},
};

#[derive(Debug, Clone, Builder)]
#[builder(on(String, into))]
pub struct MyMessageDialog {
    icon: lucide_icons::Icon,
    title: String,
    description: String,
    #[builder(default = false)]
    danger: bool,
    ok: Message,
    cancel: Option<Message>,
}

impl MyMessageDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let mut actions = row![space::horizontal()].spacing(5);

        if let Some(cancel) = &self.cancel {
            actions = actions.push(
                button("cancel")
                    .style(style::rounded_secondary_button)
                    .on_press(cancel.clone()),
            );
        }

        let mut ok_btn = button("ok").on_press(self.ok.clone());
        if self.danger {
            ok_btn = ok_btn.style(style::rounded_danger_button);
        } else {
            ok_btn = ok_btn.style(style::rounded_button);
        }

        actions = actions.push(ok_btn);

        container(
            column![
                row![Text::from(self.icon), text(&self.title)].spacing(5),
                rule::horizontal(1),
                space(),
                text(&self.description),
                space(),
                actions
            ]
            .spacing(5)
            .padding(10),
        )
        .style(style::card)
        .center(Length::Fill)
        .into()
    }
}
