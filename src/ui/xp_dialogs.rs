// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::Window;
use std::{path::PathBuf, process::Command};

#[derive(Clone, Copy, Debug)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

pub fn alert(_: &dyn Window, title: String, text: String, level: MessageLevel) -> Message {
    let level = match level {
        MessageLevel::Info => 64 + 256,
        MessageLevel::Warning => 48 + 256,
        MessageLevel::Error => 16 + 256,
    };

    let arg = format!(
        "javascript: \
            var sh=new ActiveXObject('WScript.Shell'); \
            sh.Popup('{}',0,'{}',{}); \
            WScript.Quit(0);",
        text.replace("\\", "\\\\").replace("'", "\\'"),
        title.replace("\\", "\\\\").replace("'", "\\'"),
        level
    );

    let _ = Command::new("mshta").arg(arg).status();

    Message::None
}

pub fn confirm(
    _: &dyn Window,
    title: String,
    text: String,
    level: MessageLevel,
    on_confirm: Message,
) -> Message {
    let level = match level {
        MessageLevel::Info => 64 + 1 + 256,
        MessageLevel::Warning => 48 + 1 + 256,
        MessageLevel::Error => 16 + 1 + 256,
    };

    let arg = format!(
        "javascript: \
            var sh=new ActiveXObject('WScript.Shell'); \
            var btn=sh.Popup('{}',0,'{}',{}); \
            WScript.Quit(btn);",
        text.replace("\\", "\\\\").replace("'", "\\'"),
        title.replace("\\", "\\\\").replace("'", "\\'"),
        level
    );

    let status = Command::new("mshta")
        .arg(arg)
        .status()
        .map(|s| s.code())
        .ok()
        .flatten()
        .unwrap_or(2);

    if status == 1 {
        on_confirm
    } else {
        Message::None
    }
}

pub fn pick_file(
    _: &dyn Window,
    title: String,
    _: impl IntoIterator<Item = (String, Vec<String>)>,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let arg = format!(
        "javascript: \
            var dlg = new ActiveXObject('UserAccounts.CommonDialog'); \
            dlg.Title = '{}'; \
            if (dlg.ShowOpen()) WScript.Echo(dlg.FileName); \
            close();",
        title.replace("'", "\\'").replace("\\", "\\\\"),
    );

    let Ok(output) = Command::new("mshta").arg(arg).output() else {
        return Message::None;
    };

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        Message::None
    } else {
        on_picked(PathBuf::from(path))
    }
}

pub fn pick_dir(
    _: &dyn Window,
    title: String,
    _: impl IntoIterator<Item = (String, Vec<String>)>,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let arg = format!(
        "javascript: \
            var sh = new ActiveXObject('Shell.Application'); \
            var f = sh.BrowseForFolder(0, '{}', 0); \
            if (f) WScript.Echo(f.self.Path); \
            close();",
        title.replace("\\", "\\\\").replace("'", "\\'")
    );

    let Ok(output) = Command::new("mshta").arg(arg).output() else {
        return Message::None;
    };

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        Message::None
    } else {
        on_picked(PathBuf::from(path))
    }
}

pub fn save_file(
    _: &dyn Window,
    title: String,
    _: impl IntoIterator<Item = (String, Vec<String>)>,
    filename: String,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let arg = format!(
        "javascript: \
            var dlg = new ActiveXObject('UserAccounts.CommonDialog'); \
            dlg.Title = '{}'; \
            dlg.FileName = '{}'; \
            if (dlg.ShowSave()) WScript.Echo(dlg.FileName); \
            close();",
        title.replace("'", "\\'").replace("\\", "\\\\"),
        filename.replace("'", "\\'").replace("\\", "\\\\"),
    );

    let Ok(output) = Command::new("mshta").arg(arg).output() else {
        return Message::None;
    };

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        Message::None
    } else {
        on_picked(PathBuf::from(path))
    }
}
