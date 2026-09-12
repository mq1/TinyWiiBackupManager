// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use derive_setters::Setters;
use iced::{
    Element, Length, Theme,
    widget::{button, column, row, rule, text},
};
use lucide_icons::Icon;
use std::{borrow::Cow, ffi::OsString, path::PathBuf};

pub trait ToUrl {
    fn to_url(&self) -> OsString;
}

impl ToUrl for &str {
    fn to_url(&self) -> OsString {
        OsString::from(self)
    }
}

impl ToUrl for &PathBuf {
    fn to_url(&self) -> OsString {
        self.as_os_str().to_os_string()
    }
}

impl<F: Fn() -> OsString> ToUrl for F {
    fn to_url(&self) -> OsString {
        self()
    }
}

struct Url<T>(pub T);

impl<T: ToUrl> From<T> for Url<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: ToUrl> Url<T> {
    pub fn url(&self) -> OsString {
        self.0.to_url()
    }
}

#[derive(Setters)]
pub struct MyLink<'a, T> {
    #[setters(into)]
    label: Cow<'a, str>,

    #[setters(into)]
    url: Url<T>,

    icon: Icon,
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

impl<'a, T: ToUrl + 'a> From<MyLink<'a, T>> for Element<'a, Message> {
    fn from(link: MyLink<'a, T>) -> Self {
        fn style(theme: &Theme, status: button::Status) -> button::Style {
            button::Style {
                text_color: theme.palette().primary,
                ..button::text(theme, status)
            }
        }

        button(
            column![
                row![link.icon.widget(), text(link.label)].spacing(5),
                underline()
            ]
            .width(Length::Shrink),
        )
        .style(style)
        .padding(0)
        .on_press_with(move || Message::Open(link.url.url()))
        .into()
    }
}

pub fn my_link<'a, T: ToUrl>(label: impl Into<Cow<'a, str>>, url: T) -> MyLink<'a, T> {
    MyLink {
        label: label.into(),
        url: url.into(),
        icon: Icon::Globe,
    }
}
