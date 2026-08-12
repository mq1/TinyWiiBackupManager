// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Length,
    border::Border,
    widget::{Button, Row, button, container, text},
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
    toolbar: bool,
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

    pub fn toolbar(mut self) -> Self {
        self.toolbar = true;
        self.rounded = true;
        self.kind = MyButtonKind::Primary;
        self
    }

    pub fn view(self) -> Button<'a, Message> {
        let icon = self.icon.map(|i| {
            let mut widget = i.widget();
            if self.toolbar {
                widget = widget.size(18);
            }
            widget.into()
        });

        let label = self.label.map(|l| {
            let mut widget = text(l);
            if self.toolbar {
                widget = widget.size(18);
            }
            widget.into()
        });

        let content = [icon, label]
            .into_iter()
            .flatten()
            .collect::<Row<'_, _>>()
            .spacing(5);

        let mut btn =
            button(container(content).center(Length::Shrink)).style(move |theme, status| {
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
                    radius: if self.rounded { 17. } else { 10. }.into(),
                    color: theme.extended_palette().background.weak.color,
                };

                base
            });

        if self.toolbar {
            btn = btn.padding(0).width(34).height(34);
        }

        btn
    }
}
