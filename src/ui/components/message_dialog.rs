// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::style};
use bon::Builder;
use iced::{
    Element, Length,
    widget::{Text, button, column, container, row, rule, scrollable, space, text},
};

#[derive(Debug, Clone, Builder)]
#[builder(on(String, into))]
pub struct MyMessageDialog {
    icon: lucide_icons::Icon,
    title: String,
    description: String,
    #[builder(default = false)]
    danger: bool,
    ok: Option<Message>,
}

impl MyMessageDialog {
    pub fn view(&self) -> Element<'_, Message> {
        let mut actions = row![space::horizontal()].spacing(5);

        let mut ok_btn = button("Ok");

        if let Some(ok) = &self.ok {
            actions = actions.push(
                button("Cancel")
                    .style(style::rounded_secondary_button)
                    .on_press(Message::CloseDialog),
            );

            ok_btn = ok_btn.on_press_with(|| Message::CloseDialogAndThen(Box::new(ok.clone())));
        } else {
            ok_btn = ok_btn.on_press(Message::CloseDialog);
        }

        if self.danger {
            ok_btn = ok_btn.style(style::rounded_danger_button);
        } else {
            ok_btn = ok_btn.style(style::rounded_button);
        }

        actions = actions.push(ok_btn);

        container(
            container(
                column![
                    row![Text::from(self.icon), text(&self.title)].spacing(5),
                    rule::horizontal(1),
                    space(),
                    scrollable(text(&self.description).width(Length::Fill)).height(Length::Fill),
                    space(),
                    actions
                ]
                .spacing(5)
                .padding(10),
            )
            .style(style::card)
            .center_x(600)
            .center_y(400),
        )
        .center(Length::Fill)
        .style(style::root_container)
        .into()
    }
}
