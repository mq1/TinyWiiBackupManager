// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Element, Length, Theme, border,
    widget::{Button, button, container, row, text},
};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyButtonKind {
    Secondary,
    Primary,
    Danger,
    Toolbar,
}

impl MyButtonKind {
    pub fn style(self) -> impl Fn(&Theme, button::Status) -> button::Style {
        move |theme, status| match self {
            MyButtonKind::Secondary => button::Style {
                border: border::rounded(10)
                    .width(1)
                    .color(theme.extended_palette().background.weak.color),
                ..button::subtle(theme, status)
            },
            MyButtonKind::Primary => button::Style {
                border: border::rounded(10),
                ..button::primary(theme, status)
            },
            MyButtonKind::Toolbar => button::Style {
                border: border::rounded(17),
                ..button::primary(theme, status)
            },
            MyButtonKind::Danger => button::Style {
                border: border::rounded(10),
                ..button::danger(theme, status)
            },
        }
    }
}

pub struct MyButton<'a, T> {
    label: Option<&'a str>,
    icon: Option<Icon>,
    kind: MyButtonKind,
    press: Option<T>,
    expand_width: bool,
}

impl<'a, T> MyButton<'a, T> {
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn kind(mut self, kind: MyButtonKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn expand_width(mut self) -> Self {
        self.expand_width = true;
        self
    }

    fn base(self) -> Button<'a, Message> {
        let mut btn = button(
            container(
                row![
                    self.icon.map(Icon::widget).map(|icon| match self.kind {
                        MyButtonKind::Toolbar => icon.size(18),
                        _ => icon,
                    }),
                    self.label.map(text).map(|label| match self.kind {
                        MyButtonKind::Toolbar => label.size(18),
                        _ => label,
                    })
                ]
                .spacing(5),
            )
            .center(Length::Shrink),
        )
        .style(self.kind.style());

        if self.kind == MyButtonKind::Toolbar {
            btn = btn.padding(0).width(34).height(34);
        }

        if self.expand_width {
            btn = btn.width(Length::Fill);
        }

        btn
    }
}

impl<'a> MyButton<'a, Message> {
    pub fn on_press(mut self, press: Message) -> Self {
        self.press = Some(press);
        self
    }
}

impl<'a, T: Fn() -> Message> MyButton<'a, T> {
    pub fn on_press_with(mut self, press: T) -> Self {
        self.press = Some(press);
        self
    }
}

impl<'a> From<MyButton<'a, Message>> for Element<'a, Message> {
    fn from(mut value: MyButton<'a, Message>) -> Self {
        if let Some(press) = value.press.take() {
            value.base().on_press(press).into()
        } else {
            value.base().into()
        }
    }
}

impl<'a, T: Fn() -> Message + 'a> From<MyButton<'a, T>> for Element<'a, Message> {
    fn from(mut value: MyButton<'a, T>) -> Self {
        if let Some(press) = value.press.take() {
            value.base().on_press_with(press).into()
        } else {
            value.base().into()
        }
    }
}

pub fn my_button<'a, T>() -> MyButton<'a, T> {
    MyButton {
        label: None,
        icon: None,
        kind: MyButtonKind::Secondary,
        press: None,
        expand_width: false,
    }
}
