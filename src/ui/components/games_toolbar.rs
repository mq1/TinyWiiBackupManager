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
    Alignment, Element, Font,
    widget::{button, checkbox, container, row, rule, space, text_input},
};
use lucide_icons::iced::{icon_folder_plus, icon_plus, icon_rotate_cw, icon_search};

pub fn view(state: &State) -> Element<'_, Message> {
    let mut row = row![
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
        space::horizontal()
    ];

    if state.games_filter.is_empty() {
        row = row
            .push(
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
                    .padding(10),
                )
                .style(style::card),
            )
            .push(components::sort_by::view(state));
    }

    row.push(components::view_as::view(state))
        .push(space())
        .push(my_tooltip::view(
            button(icon_rotate_cw().size(17).center())
                .width(35)
                .height(35)
                .style(style::rounded_button)
                .on_press(Message::RefreshGamesAndApps),
            "Refresh games",
        ))
        .push(my_tooltip::view(
            button(icon_plus().size(17).center())
                .width(35)
                .height(35)
                .style(style::rounded_button)
                .on_press(Message::PickGames),
            "Add games",
        ))
        .push(my_tooltip::view(
            button(icon_folder_plus().size(17).center())
                .width(35)
                .height(35)
                .style(style::rounded_button)
                .on_press(Message::ChooseGamesSrcDir),
            "Add games recursively",
        ))
        .spacing(10)
        .padding(10)
        .align_y(Alignment::Center)
        .into()
}
