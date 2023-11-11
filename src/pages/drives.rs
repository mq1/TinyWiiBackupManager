// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

use iced::widget::{button, horizontal_space, pick_list, vertical_space, Column, Row, Text};
use iced::{Alignment, Element, Length};

use crate::types::drive::Drive;
use crate::types::message::Message;
use crate::TinyWiiBackupManager;

pub fn view(app: &TinyWiiBackupManager) -> Element<Message> {
    let menu_bar = Row::new()
        .push(horizontal_space(Length::Fill))
        .push(button("Check for updates").on_press(Message::CheckForUpdates));

    let drives = Drive::list();
    let drive_picker = pick_list(drives, app.selected_drive.clone(), Message::SelectDrive);

    let chooser = Row::new()
        .push(drive_picker)
        .push(button("Open").on_press(Message::OpenDrive))
        .spacing(8);

    Column::new()
        .push(menu_bar)
        .push(vertical_space(Length::Fill))
        .push(Text::new("Choose a drive"))
        .push(chooser)
        .push(vertical_space(Length::Fill))
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .spacing(8)
        .into()
}
