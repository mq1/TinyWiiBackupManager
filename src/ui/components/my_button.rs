// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use derive_setters::Setters;
use iced::{
    Element, Length, Theme, border,
    widget::{button, container, row, text},
};
use lucide_icons::Icon;
use tap::Pipe;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MyButtonKind {
    #[default]
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

trait Press {
    fn msg(&self) -> Message;
}

impl Press for Message {
    fn msg(&self) -> Message {
        self.clone()
    }
}

impl<T: Fn() -> Message> Press for T {
    fn msg(&self) -> Message {
        self()
    }
}

#[derive(Setters)]
pub struct MyButton<'a, T: Press> {
    #[setters(strip_option)]
    label: Option<&'a str>,

    #[setters(strip_option)]
    icon: Option<Icon>,

    kind: MyButtonKind,

    #[setters(skip)]
    press: Option<T>,

    #[setters(bool)]
    expand_width: bool,
}

impl<'a> MyButton<'a, Message> {
    pub fn on_press(mut self, press: Message) -> Self {
        self.press = Some(press);
        self
    }

    pub fn on_press_maybe(mut self, press: Option<Message>) -> Self {
        self.press = press;
        self
    }
}

impl<'a, T: Fn() -> Message> MyButton<'a, T> {
    pub fn on_press_with(mut self, press: T) -> Self {
        self.press = Some(press);
        self
    }
}

impl<'a, T: Press + 'a> From<MyButton<'a, T>> for Element<'a, Message> {
    fn from(value: MyButton<'a, T>) -> Self {
        button(
            container(
                row![
                    value.icon.map(Icon::widget).map(|icon| match value.kind {
                        MyButtonKind::Toolbar => icon.size(18),
                        _ => icon,
                    }),
                    value.label.map(text).map(|label| match value.kind {
                        MyButtonKind::Toolbar => label.size(18),
                        _ => label,
                    })
                ]
                .spacing(5),
            )
            .center(Length::Shrink),
        )
        .style(value.kind.style())
        .pipe(|btn| match value.kind {
            MyButtonKind::Toolbar => btn.padding(0).width(34).height(34),
            _ => btn,
        })
        .pipe(|btn| match value.press {
            Some(press) => btn.on_press_with(move || press.msg()),
            None => btn,
        })
        .pipe(|btn| match value.expand_width {
            true => btn.width(Length::Fill),
            false => btn,
        })
        .into()
    }
}

pub fn my_button<'a, T: Press>() -> MyButton<'a, T> {
    MyButton {
        label: None,
        icon: None,
        kind: MyButtonKind::Secondary,
        press: None,
        expand_width: false,
    }
}
