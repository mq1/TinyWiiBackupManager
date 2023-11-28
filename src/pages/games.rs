// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use iced::widget::{button, checkbox, column, horizontal_rule, row, scrollable, text, Checkbox};
use iced::Element;
use iced_aw::Spinner;

use crate::types::drive::Drive;
use crate::types::message::Message;
use crate::TinyWiiBackupManager;

pub fn view<'a>(app: &'a TinyWiiBackupManager, drive: &'a Drive) -> Element<'a, Message> {
    let content: Element<Message> = if let Some(games) = &app.games {
        let mut content = column![].spacing(4);

        for (i, (game, checked)) in games.iter().cloned().enumerate() {
            let gib = game.size as f32 / 1024. / 1024. / 1024.;
            let text = format!("{}: {} ({:.2} GiB)", game.id, game.display_title, gib);

            let checkbox: Checkbox<Message> = checkbox(text, checked, move |selected| {
                Message::SelectGame(i, selected)
            });

            content = content.push(checkbox);

            if i < games.len() - 1 {
                content = content.push(horizontal_rule(1));
            }
        }

        let content = scrollable(content);

        let actions = row![
            button("Add games").on_press(Message::AddGames(drive.clone())),
            button("Remove selected games").on_press(Message::RemoveGames),
        ]
        .spacing(8);

        column![actions, content].spacing(8).into()
    } else {
        Spinner::new().into()
    };

    column![
        text(&drive.name).size(30),
        text(format!(
            "{}/{} GiB",
            drive.available_space, drive.total_space
        )),
        content,
    ]
    .spacing(8)
    .padding(8)
    .into()
}
