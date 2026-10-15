// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, ui::components::my_card::my_card};
use iced::{
    Element, Length, Theme,
    widget::{button, column, row, rule, text, text::IntoFragment, tooltip},
};
use lucide_iced::{bytes::GLOBE, themed_icon};
use std::ffi::{OsStr, OsString};

pub fn make_preview(url: &str) -> String {
    if url.len() > 63 {
        let mut preview = String::with_capacity(63);

        preview.push_str(&url[..30]);
        preview.push_str("..."); // 3 chars
        preview.push_str(&url[url.len() - 30..]);

        preview
    } else {
        url.to_string()
    }
}

#[derive(Clone)]
pub struct MyLink<L, U>
where
    L: Clone,
    U: Clone,
{
    label: L,
    url: U,
    url_preview: String,
    icon_bytes: &'static [u8],
}

impl<'a, L, U> MyLink<L, U>
where
    L: IntoFragment<'a> + Clone,
    U: AsRef<OsStr> + Into<OsString> + 'a + Clone,
{
    pub fn new(label: L, url: U) -> Self {
        Self {
            label,
            url_preview: make_preview(&url.as_ref().to_string_lossy()),
            url,
            icon_bytes: GLOBE,
        }
    }

    pub fn icon(mut self, icon_bytes: &'static [u8]) -> Self {
        self.icon_bytes = icon_bytes;
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
    L: IntoFragment<'a> + Clone,
    U: Into<OsString> + 'a + Clone,
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
                    row![themed_icon(link.icon_bytes, 14.), text(link.label)].spacing(5),
                    underline()
                ]
                .width(Length::Shrink),
            )
            .style(style)
            .padding(0)
            .on_press_with(move || Message::Open(link.url.clone().into())),
            my_card(text(link.url_preview)),
            tooltip::Position::FollowCursor,
        )
        .into()
    }
}

pub fn my_link<'a, L, U>(label: L, url: U) -> MyLink<L, U>
where
    L: IntoFragment<'a> + Clone,
    U: Into<OsString> + AsRef<OsStr> + 'a + Clone,
{
    MyLink::new(label, url)
}
