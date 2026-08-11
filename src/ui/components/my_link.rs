// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element, Length,
    widget::{button, column, container, row, text, text::IntoFragment},
};
use lucide_icons::Icon;
use std::{ffi::OsString, path::PathBuf};

pub trait Url {
    fn get(&self) -> OsString;
}

impl Url for &PathBuf {
    fn get(&self) -> OsString {
        self.into()
    }
}

impl Url for &str {
    fn get(&self) -> OsString {
        self.into()
    }
}

impl<F, O> Url for F
where
    O: Into<OsString>,
    F: Fn() -> O,
{
    fn get(&self) -> OsString {
        self().into()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MyLink<L, U> {
    label: L,
    icon: Option<Icon>,
    url: U,
}

impl<'a, L, U> MyLink<L, U>
where
    L: IntoFragment<'a>,
    U: Url + 'a,
{
    pub fn new(label: L, url: U) -> Self {
        Self {
            label,
            url,
            icon: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn view(self) -> Element<'a, Message> {
        let icon = self.icon.unwrap_or(Icon::Globe).widget();
        let label = row![icon, text(self.label)].spacing(5);

        let underline = container(row![].height(1).width(Length::Fill)).style(|theme| {
            let mut base = container::bordered_box(theme);
            base.border.color = theme.palette().primary;
            base
        });

        button(column![label, underline].width(Length::Shrink))
            .style(|theme, status| {
                let mut base = button::text(theme, status);
                base.text_color = theme.palette().primary;
                base
            })
            .padding(0)
            .on_press_with(move || Message::Open(self.url.get()))
            .into()
    }
}
