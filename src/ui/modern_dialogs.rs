// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::Window;
use native_dialog::DialogBuilder;
pub use native_dialog::MessageLevel;
use std::path::PathBuf;

pub fn confirm(
    window: &dyn Window,
    title: String,
    text: String,
    level: MessageLevel,
    on_confirm: Message,
) -> Message {
    let dialog = DialogBuilder::message()
        .set_owner(&window)
        .set_title(title)
        .set_text(text)
        .set_level(level)
        .confirm();

    if dialog.show().unwrap_or(false) {
        on_confirm
    } else {
        Message::None
    }
}

pub fn alert(
    window: &dyn Window,
    title: String,
    text: Option<String>,
    level: MessageLevel,
) -> Message {
    let mut dialog = DialogBuilder::message()
        .set_owner(&window)
        .set_title(title)
        .set_text(text)
        .set_level(level)
        .alert();

    let _ = dialog.show();
    Message::None
}

pub fn pick_dir(
    window: &dyn Window,
    title: String,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let dialog = DialogBuilder::file()
        .set_owner(&window)
        .set_title(title)
        .open_single_dir();

    if let Some(path) = dialog.show().unwrap_or(None) {
        on_picked(path)
    } else {
        Message::None
    }
}

pub fn pick_file(
    window: &dyn Window,
    title: String,
    filters: impl IntoIterator<Item = (String, Vec<String>)>,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let dialog = DialogBuilder::file()
        .set_owner(&window)
        .set_title(title)
        .add_filters(filters)
        .open_single_file();

    if let Some(path) = dialog.show().unwrap_or(None) {
        on_picked(path)
    } else {
        Message::None
    }
}

pub fn pick_files(
    window: &dyn Window,
    title: String,
    filters: impl IntoIterator<Item = (String, Vec<String>)>,
    on_picked: impl FnOnce(Vec<PathBuf>) -> Message + 'static,
) -> Message {
    let dialog = DialogBuilder::file()
        .set_owner(&window)
        .set_title(title)
        .add_filters(filters)
        .open_multiple_file();

    let paths = dialog.show().unwrap_or_default();
    if paths.is_empty() {
        Message::None
    } else {
        on_picked(paths)
    }
}

pub fn save_file(
    window: &dyn Window,
    title: String,
    filters: impl IntoIterator<Item = (String, Vec<String>)>,
    filename: String,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let dialog = DialogBuilder::file()
        .set_owner(&window)
        .set_title(title)
        .add_filters(filters)
        .set_filename(filename)
        .save_single_file();

    if let Some(path) = dialog.show().unwrap_or(None) {
        on_picked(path)
    } else {
        Message::None
    }
}
