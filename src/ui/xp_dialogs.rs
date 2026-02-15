// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::message::Message;
use iced::Window;
use std::process::Command;

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
        "javascript:var sh=new ActiveXObject('WScript.Shell'); \
         sh.Popup('{}',0,'{}',{}); \
         close();",
        text.replace("'", "\\'").replace("\\", "\\\\"),
        title.replace("'", "\\'").replace("\\", "\\\\"),
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
        "javascript:var sh=new ActiveXObject('WScript.Shell'); \
         var btn=sh.Popup('{}',0,'{}',{}); \
         WScript.Quit(btn);",
        text.replace("'", "\\'").replace("\\", "\\\\"),
        title.replace("'", "\\'").replace("\\", "\\\\"),
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
