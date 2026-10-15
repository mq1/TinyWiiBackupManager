// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::ui::components::my_card::my_card;
use iced::{
    Element,
    widget::{column, row, rule, space},
};
use lucide_iced::themed_icon;

pub struct MyGroup<'a, Msg> {
    pub title: &'a str,
    pub icon_bytes: Option<&'static [u8]>,
    pub content: Element<'a, Msg>,
    pub top_right_content: Option<Element<'a, Msg>>,
}

impl<'a, Msg> MyGroup<'a, Msg> {
    pub fn new(title: &'a str, content: impl Into<Element<'a, Msg>>) -> Self {
        Self {
            title,
            icon_bytes: None,
            top_right_content: None,
            content: content.into(),
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn icon(mut self, icon_bytes: &'static [u8]) -> Self {
        self.icon_bytes = Some(icon_bytes);
        self
    }

    pub fn top_right_content(mut self, content: impl Into<Element<'a, Msg>>) -> Self {
        self.top_right_content = Some(content.into());
        self
    }

    pub fn content(mut self, content: impl Into<Element<'a, Msg>>) -> Self {
        self.content = content.into();
        self
    }
}

impl<'a, Msg> From<MyGroup<'a, Msg>> for Element<'a, Msg>
where
    Msg: Clone + 'a,
{
    fn from(group: MyGroup<'a, Msg>) -> Element<'a, Msg> {
        my_card(
            column![
                row![
                    group.icon_bytes.map(|icon| themed_icon(icon, 14.)),
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

pub fn my_group<'a, Msg>(title: &'a str, content: impl Into<Element<'a, Msg>>) -> MyGroup<'a, Msg> {
    MyGroup::new(title, content)
}
