// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, ui::components::my_card::my_card};
use iced::{
    Element, Length, Theme,
    widget::{button, column, row, rule, text, tooltip},
};
use lucide_icons::Icon;
use std::ffi::{OsStr, OsString};

pub fn make_preview<'a>(url: &str) -> String {
    if url.len() > 63 {
        let mut preview = String::with_capacity(33);

        preview.push_str(&url[..30]);
        preview.push_str("..."); // 3 chars
        preview.push_str(&url[url.len() - 30..]);

        preview
    } else {
        url.to_string()
    }
}

pub struct MyLink<L, U> {
    label: L,
    url: U,
    url_preview: String,
    icon: Icon,
}

impl<'a, L, U> MyLink<L, U>
where
    L: Into<String> + 'a,
    U: AsRef<OsStr> + Into<OsString> + 'a,
{
    pub fn new(label: L, url: U) -> Self {
        Self {
            label,
            url_preview: make_preview(&url.as_ref().to_string_lossy()),
            url,
            icon: Icon::Globe,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }
}

fn underline() -> Element<'static, Message> {
    fn style(theme: &Theme) -> rule::Style {
        rule::Style {
            color: theme.palette().primary,
            ..rule::default(theme)
        }
    }

    rule::horizontal(1).style(style).into()
}

impl<'a, L, U> From<MyLink<L, U>> for Element<'a, Message>
where
    L: Into<String> + 'a,
    U: Into<OsString> + 'a,
{
    fn from(link: MyLink<L, U>) -> Self {
        fn style(theme: &Theme, status: button::Status) -> button::Style {
            button::Style {
                text_color: theme.palette().primary,
                ..button::text(theme, status)
            }
        }

        tooltip(
            button(
                column![
                    row![link.icon.widget(), text(link.label.into())].spacing(5),
                    underline()
                ]
                .width(Length::Shrink),
            )
            .style(style)
            .padding(0)
            .on_press(Message::Open(link.url.into())),
            my_card(text(link.url_preview)),
            tooltip::Position::FollowCursor,
        )
        .into()
    }
}

pub fn my_link<'a, L, U>(label: L, url: U) -> MyLink<L, U>
where
    L: Into<String> + 'a,
    U: AsRef<OsStr> + Into<OsString> + 'a,
{
    MyLink::new(label, url)
}
