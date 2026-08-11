// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Alignment, Length,
    advanced::Widget,
    border::Border,
    widget::{Button, Row, button, column, text},
};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum MyButtonKind {
    #[default]
    Secondary,
    Primary,
    Danger,
}

#[derive(Debug, Clone, Default)]
pub struct MyButton<'a> {
    label: Option<&'a str>,
    icon: Option<Icon>,
    kind: MyButtonKind,
    rounded: bool,
}

impl<'a> MyButton<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn primary(mut self) -> Self {
        self.kind = MyButtonKind::Primary;
        self
    }

    pub fn danger(mut self) -> Self {
        self.kind = MyButtonKind::Danger;
        self
    }

    pub fn rounded(mut self) -> Self {
        self.rounded = true;
        self
    }

    pub fn view(self) -> Button<'a, Message> {
        let content = [
            self.icon.map(|i| i.widget().into()),
            self.label.map(|l| text(l).into()),
        ]
        .into_iter()
        .flatten()
        .collect::<Row<'_, _>>()
        .spacing(5);

        let content_width = content.size_hint().width;

        button(
            column![content]
                .align_x(Alignment::Center)
                .width(Length::Fill),
        )
        .style(move |theme, status| {
            let mut base = match self.kind {
                MyButtonKind::Secondary => button::subtle(theme, status),
                MyButtonKind::Primary => button::primary(theme, status),
                MyButtonKind::Danger => button::danger(theme, status),
            };

            base.border = Border {
                width: if self.kind == MyButtonKind::Secondary {
                    1.0
                } else {
                    0.0
                },
                radius: if self.rounded { 90. } else { 12. }.into(),
                color: theme.extended_palette().background.weak.color,
            };

            base
        })
        .width(content_width)
    }
}
