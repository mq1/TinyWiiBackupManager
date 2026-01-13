// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    message::Message,
    state::State,
    ui::{
        components::{self, my_tooltip},
        style,
    },
};
use iced::{
    Alignment, Element, Font, font,
    widget::{button, checkbox, container, row, rule, space, text_input},
};
use lucide_icons::iced::{icon_box, icon_plus, icon_pointer, icon_rotate_cw, icon_search};

pub fn view(state: &State) -> Element<'_, Message> {
    row![
        container(
            row![
                space(),
                icon_search(),
                text_input("Search by Title/ID", &state.games_filter)
                    .width(200)
                    .style(style::search_bar)
                    .on_input(Message::UpdateGamesFilter),
            ]
            .spacing(10)
            .padding(5)
            .align_y(Alignment::Center)
        )
        .style(style::card),
        space::horizontal(),
        container(
            row![
                my_tooltip::view(
                    checkbox(state.show_wii)
                        .label(lucide_icons::Icon::Pointer.unicode())
                        .font(Font::with_name("lucide"))
                        .style(style::toolbar_checkbox)
                        .on_toggle(Message::ShowWii),
                    "Show Wii Games"
                ),
                space(),
                rule::vertical(1),
                space(),
                my_tooltip::view(
                    checkbox(state.show_gc)
                        .label(lucide_icons::Icon::Box.unicode())
                        .font(Font::with_name("lucide"))
                        .style(style::toolbar_checkbox)
                        .on_toggle(Message::ShowGc),
                    "Show GameCube Games"
                ),
            ]
            .height(38)
            .spacing(5)
            .padding(10)
        )
        .style(style::card),
        components::sort_by::view(state),
        space(),
        my_tooltip::view(
            button(icon_rotate_cw().size(18).center())
                .width(35)
                .height(35)
                .style(style::rounded_button)
                .on_press(Message::RefreshGamesAndApps),
            "Refresh Games"
        ),
        my_tooltip::view(
            button(icon_plus().size(18).center())
                .width(35)
                .height(35)
                .style(style::rounded_button)
                .on_press(Message::ChooseGamesToAdd),
            "Add Games"
        )
    ]
    .spacing(10)
    .padding(10)
    .align_y(Alignment::Center)
    .into()
}
