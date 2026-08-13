// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::messages::Message;
use iced::{
    Length,
    border::Border,
    widget::{Button, Row, button, container, text},
};
use lucide_icons::Icon;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MyButtonKind {
    Secondary,
    Primary,
    Danger,
    Toolbar,
}

pub fn my_button<'a, L, I>(label: L, icon: I, kind: MyButtonKind) -> Button<'a, Message>
where
    L: Into<Option<&'a str>>,
    I: Into<Option<Icon>>,
{
    let mut content = Row::new().spacing(5);

    if let Some(icon) = icon.into() {
        let mut widget = icon.widget();
        if kind == MyButtonKind::Toolbar {
            widget = widget.size(18);
        }
        content = content.push(widget);
    }

    if let Some(label) = label.into() {
        let mut widget = text(label);
        if kind == MyButtonKind::Toolbar {
            widget = widget.size(18);
        }
        content = content.push(widget);
    };

    let mut btn = button(container(content).center(Length::Shrink)).style(move |theme, status| {
        let mut base = match kind {
            MyButtonKind::Secondary => button::subtle(theme, status),
            MyButtonKind::Primary | MyButtonKind::Toolbar => button::primary(theme, status),
            MyButtonKind::Danger => button::danger(theme, status),
        };

        base.border = Border {
            width: if kind == MyButtonKind::Secondary {
                1.0
            } else {
                0.0
            },
            radius: if kind == MyButtonKind::Toolbar {
                17.
            } else {
                10.
            }
            .into(),
            color: theme.extended_palette().background.weak.color,
        };

        base
    });

    if kind == MyButtonKind::Toolbar {
        btn = btn.padding(0).width(34).height(34);
    }

    btn
}
