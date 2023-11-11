// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

use iced::widget::{button, checkbox, horizontal_rule, scrollable, text, Checkbox, Column, Row};
use iced::Element;

use crate::types::drive::Drive;
use crate::types::message::Message;
use crate::TinyWiiBackupManager;

pub fn view<'a>(app: &'a TinyWiiBackupManager, drive: &'a Drive) -> Element<'a, Message> {
    let mut content = Column::new().spacing(4);

    for (i, (game, checked)) in app.games.iter().cloned().enumerate() {
        let gib = game.size as f32 / 1024. / 1024. / 1024.;
        let text = format!("{}: {} ({:.2} GiB)", game.id, game.display_title, gib);

        let checkbox: Checkbox<Message> = checkbox(text, checked, move |selected| {
            Message::SelectGame(i, selected)
        });

        content = content.push(checkbox);

        if i < app.games.len() - 1 {
            content = content.push(horizontal_rule(1));
        }
    }

    let content = scrollable(content);

    let actions = Row::new()
        .push(button("Add games").on_press(Message::AddGames(drive.clone())))
        .push(button("Remove selected games").on_press(Message::RemoveGames))
        .spacing(8);

    Column::new()
        .push(text(&drive.name).size(30))
        .push(text(format!(
            "{}/{} GiB",
            drive.available_space, drive.total_space
        )))
        .push(actions)
        .push(content)
        .spacing(8)
        .padding(8)
        .into()
}
