// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use iced::widget::{button, column, horizontal_space, pick_list, row, vertical_space, Text};
use iced::{Alignment, Element, Length};

use crate::types::drive::Drive;
use crate::types::message::Message;
use crate::TinyWiiBackupManager;

pub fn view(app: &TinyWiiBackupManager) -> Element<Message> {
    let updates_button = if app.checking_for_updates {
        button("Checking for updates...")
    } else {
        button("Check for updates").on_press(Message::CheckForUpdates)
    };

    let menu_bar = row![horizontal_space(Length::Fill), updates_button];

    let drives = Drive::list();
    let drive_picker = pick_list(drives, app.selected_drive.clone(), Message::SelectDrive);

    let chooser = row![drive_picker, button("Open").on_press(Message::OpenDrive)].spacing(8);

    column![
        menu_bar,
        vertical_space(Length::Fill),
        Text::new("Choose a drive"),
        chooser,
        vertical_space(Length::Fill),
    ]
    .align_items(Alignment::Center)
    .width(Length::Fill)
    .spacing(8)
    .into()
}
