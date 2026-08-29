// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    messages::Message,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
        my_link::my_link,
    },
};
use iced::{
    Alignment, Element,
    widget::{column, image, row, rule, space, text, tooltip},
};
use lucide_icons::{
    Icon,
    iced::{icon_file_question, icon_gamepad, icon_globe, icon_notebook_pen, icon_pin, icon_tag},
};

pub fn game_info<'a>(
    game: &'a Game,
    disc_info: Option<&'a wii_disc_info::Meta>,
) -> Element<'a, Message> {
    let content: Element<'a, _> = if let Some(disc_info) = disc_info {
        let cover: Element<'_, Message> = match &game.cover {
            Some(cover) => {
                let (w, h) = cover.dimensions();
                let bytes = cover.as_raw().clone();
                let handle = iced::widget::image::Handle::from_rgba(w, h, bytes);
                image(handle).height(200).into()
            }
            None => space().height(200).into(),
        };

        row![
            column![
                row![
                    icon_file_question(),
                    text!("Format: {}", disc_info.format())
                ]
                .spacing(5),
                row![icon_tag(), text!("Game ID: {}", disc_info.game_id())].spacing(5),
                row![
                    icon_notebook_pen(),
                    text!("Game Title: {}", disc_info.game_title())
                ]
                .spacing(5),
                row![
                    icon_globe(),
                    text!("Region: {}", disc_info.game_id().region())
                ]
                .spacing(5),
                row![
                    icon_gamepad(),
                    if disc_info.is_wii() {
                        text("System: Wii")
                    } else {
                        text("System: GameCube")
                    }
                ]
                .spacing(5),
                row![
                    icon_pin(),
                    text!("Disc Version: {}", disc_info.disc_version())
                ]
                .spacing(5),
            ]
            .spacing(5),
            space::horizontal(),
            cover,
        ]
        .padding(20)
        .align_y(Alignment::Center)
        .into()
    } else {
        text("No disc info available").center().into()
    };

    my_card(
        column![
            column![
                text(&game.title).size(18),
                my_link(game.path.to_string_lossy(), || &game.path, Icon::Folder)
            ]
            .spacing(10)
            .padding(20),
            space::vertical(),
            content,
            space::vertical(),
            rule::horizontal(1),
            row![
                space::horizontal(),
                tooltip(
                    my_button("SHA1", Icon::SearchCheck, MyButtonKind::Secondary)
                        .on_press_with(|| Message::CalcGameSha1(game.clone())),
                    my_card("Check if your dump is 100% byte identical to the Redump one"),
                    tooltip::Position::Bottom
                ),
                my_button("Close", None, MyButtonKind::Secondary).on_press(Message::CloseModal)
            ]
            .spacing(10)
            .padding(10)
        ]
        .width(600)
        .height(400),
    )
    .padding(0)
    .into()
}
