// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{config::ViewAs, messages::Message, state::AppState, ui::my_palette};
use iced::{
    Background, Element, border,
    widget::{button, row},
};
use lucide_icons::iced::{icon_layout_grid, icon_table};

pub fn view(state: &AppState) -> Element<'_, Message> {
    row![
        button(icon_layout_grid())
            .on_press(Message::ViewAsGrid)
            .style(|theme, status| {
                let mut base = button::text(theme, status);
                let bg = match state.config.contents.view_as {
                    ViewAs::Grid => Background::Color(theme.palette().primary.scale_alpha(0.5)),
                    ViewAs::Table => my_palette::card_bg(theme),
                };
                base.background = Some(bg);
                base.border.radius = border::radius(0).top_left(10).bottom_left(10);
                base
            }),
        button(icon_table())
            .on_press(Message::ViewAsTable)
            .style(|theme, status| {
                let mut base = button::text(theme, status);
                let bg = match state.config.contents.view_as {
                    ViewAs::Table => Background::Color(theme.palette().primary.scale_alpha(0.5)),
                    ViewAs::Grid => my_palette::card_bg(theme),
                };
                base.background = Some(bg);
                base.border.radius = border::radius(0).top_right(10).bottom_right(10);
                base
            })
    ]
    .into()
}
