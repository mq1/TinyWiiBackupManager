// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::game::Game,
    long_operation::LongOperationKind,
    message::Message,
    state::AppState,
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
    scrubbable: bool,
    state: &'a AppState,
) -> Element<'a, Message> {
    let content: Element<'a, _> = if let Some(disc_info) = disc_info {
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
            game.cover().map(|cover| image(cover).height(200)),
        ]
        .padding(20)
        .align_y(Alignment::Center)
        .into()
    } else {
        text("No disc info available").center().into()
    };

    let scrub_btn = scrubbable.then(|| {
        let mut btn = my_button()
            .label("Scrub")
            .icon(Icon::Shredder)
            .kind(MyButtonKind::Secondary);

        if state.long_operations.is_idle(LongOperationKind::Scrub) {
            btn = btn.on_press_with(|| Message::AskConfirmScrub(game.clone()));
        }

        tooltip(
            btn,
            my_card("Remove the update partition"),
            tooltip::Position::Bottom,
        )
    });

    my_card(
        column![
            column![
                text(game.title()).size(18),
                my_link(game.path().to_string_lossy(), game.path()).icon(Icon::Folder)
            ]
            .spacing(10)
            .padding(20),
            space::vertical(),
            content,
            space::vertical(),
            rule::horizontal(1),
            row![
                space(),
                my_link("GameTDB Page", game.gametdb_url()),
                space::horizontal(),
                scrub_btn,
                tooltip(
                    my_button()
                        .label("Get cheats")
                        .icon(Icon::Skull)
                        .kind(MyButtonKind::Secondary)
                        .on_press_with(|| Message::DownloadTxtCodes(game.clone())),
                    my_card("Download txtcodes (source configurable in settings)"),
                    tooltip::Position::Bottom
                ),
                tooltip(
                    my_button()
                        .label("SHA1")
                        .icon(Icon::SearchCheck)
                        .kind(MyButtonKind::Secondary)
                        .on_press_with(|| Message::CalcGameSha1(game.clone())),
                    my_card("Check if your dump is 100% byte identical to the Redump one"),
                    tooltip::Position::Bottom
                ),
                my_button()
                    .label("Close")
                    .kind(MyButtonKind::Primary)
                    .on_press(Message::CloseModal)
            ]
            .spacing(10)
            .padding(10)
            .align_y(Alignment::Center)
        ]
        .width(600)
        .height(400),
    )
    .padding(0)
    .into()
}
