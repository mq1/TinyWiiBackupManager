// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_card::my_card};
use iced::{
    Element,
    widget::{column, row, rule, space},
};
use lucide_icons::Icon;

pub struct MyGroup<'a> {
    pub title: &'a str,
    pub icon: Option<Icon>,
    pub top_right_content: Option<Element<'a, Message>>,
    pub content: Option<Element<'a, Message>>,
}

impl<'a> MyGroup<'a> {
    pub fn new(title: &'a str) -> Self {
        MyGroup {
            title,
            icon: None,
            top_right_content: None,
            content: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn top_right_content(mut self, top_right_content: impl Into<Element<'a, Message>>) -> Self {
        self.top_right_content = Some(top_right_content.into());
        self
    }

    pub fn content(mut self, content: impl Into<Element<'a, Message>>) -> Self {
        self.content = Some(content.into());
        self
    }
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

pub fn my_group<'a>(title: &'a str) -> MyGroup<'a> {
    MyGroup::new(title)
}
