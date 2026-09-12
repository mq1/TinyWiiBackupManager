// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_card::my_card};
use derive_setters::Setters;
use iced::{
    Element,
    widget::{column, row, rule, space},
};
use lucide_icons::Icon;

#[derive(Default, Setters)]
pub struct MyGroup<'a> {
    pub title: &'a str,

    #[setters(strip_option, into)]
    pub icon: Option<Icon>,

    #[setters(strip_option, into)]
    pub top_right_content: Option<Element<'a, Message>>,

    #[setters(strip_option, into)]
    pub content: Option<Element<'a, Message>>,
}

impl<'a> From<MyGroup<'a>> for Element<'a, Message> {
    fn from(group: MyGroup<'a>) -> Element<'a, Message> {
        my_card(
            column![
                row![
                    group.icon.map(|icon| icon.widget()),
                    group.title,
                    space::horizontal(),
                    group.top_right_content
                ]
                .spacing(5),
                rule::horizontal(1),
                group.content
            ]
            .spacing(10)
            .padding(5),
        )
        .padding(5)
        .into()
    }
}

pub fn my_group<'a>(title: &'a str, content: impl Into<Element<'a, Message>>) -> MyGroup<'a> {
    MyGroup::default().title(title).content(content)
}
