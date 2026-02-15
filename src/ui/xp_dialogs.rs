// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::needless_pass_by_value)]

use crate::message::Message;
use iced::Window;
use std::{path::PathBuf, process::Command};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

fn get_filter_string(filters: impl IntoIterator<Item = (String, Vec<String>)>) -> String {
    filters
        .into_iter()
        .map(|(name, extensions)| {
            format!(
                "{}|{}",
                name,
                extensions
                    .into_iter()
                    .map(|ext| format!("*.{ext}"))
                    .collect::<Vec<_>>()
                    .join(";")
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn alert(_: &dyn Window, title: String, text: String, level: MessageLevel) -> Message {
    let level = match level {
        MessageLevel::Info => 64,
        MessageLevel::Warning => 48,
        MessageLevel::Error => 16,
    };

    // Escape for VBScript: double quotes become doubled
    let text_escaped = text.replace('"', "\"\"");
    let title_escaped = title.replace('"', "\"\"");

    let arg = format!(
        "vbscript:Execute(\"MsgBox \"\"{text_escaped}\"\",{level},\"\"{title_escaped}\"\": close\")"
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
        MessageLevel::Info => 64 + 1,
        MessageLevel::Warning => 48 + 1,
        MessageLevel::Error => 16 + 1,
    };

    // Escape for VBScript: double quotes become doubled
    let text_escaped = text.replace('"', "\"\"");
    let title_escaped = title.replace('"', "\"\"");

    let arg = format!(
        "vbscript:Execute(\"Dim result: result = MsgBox(\"\"{text_escaped}\"\",{level},\"\"{title_escaped}\"\"): WScript.Quit(result)\")",
    );

    let output = match Command::new("mshta").arg(arg).output() {
        Ok(o) => o,
        Err(e) => {
            return Message::GenericError(e.to_string());
        }
    };

    let status = match String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
    {
        Ok(s) => s,
        Err(e) => {
            return Message::GenericError(e.to_string());
        }
    };

    if status == 1 {
        on_confirm
    } else {
        Message::None
    }
}

pub fn pick_file(
    _: &dyn Window,
    title: String,
    filters: impl IntoIterator<Item = (String, Vec<String>)>,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let title_escaped = title.replace('"', "\"\"");
    let filter_str = get_filter_string(filters);
    let filter_escaped = filter_str.replace('"', "\"\"");

    let arg = format!(
        "vbscript:Execute(\"Dim dlg: Set dlg = CreateObject(\"\"UserAccounts.CommonDialog\"\"): dlg.Title = \"\"{title_escaped}\"\": dlg.Filter = \"\"{filter_escaped}\"\": If dlg.ShowOpen Then WScript.StdOut.Write dlg.FileName: close\")"
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

pub fn pick_files(
    _: &dyn Window,
    title: String,
    filters: impl IntoIterator<Item = (String, Vec<String>)>,
    on_picked: impl FnOnce(Vec<PathBuf>) -> Message + 'static,
) -> Message {
    let title_escaped = title.replace('"', "\"\"");
    let filter_str = get_filter_string(filters);
    let filter_escaped = filter_str.replace('"', "\"\"");

    let arg = format!(
        "vbscript:Execute(\"Dim dlg: Set dlg = CreateObject(\"\"UserAccounts.CommonDialog\"\"): dlg.Title = \"\"{title_escaped}\"\": dlg.Filter = \"\"{filter_escaped}\"\": If dlg.ShowOpen Then WScript.StdOut.Write dlg.FileName: close\")"
    );

    let Ok(output) = Command::new("mshta").arg(arg).output() else {
        return Message::None;
    };

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        Message::None
    } else {
        on_picked(vec![PathBuf::from(path)])
    }
}

pub fn pick_dir(
    _: &dyn Window,
    title: String,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let title_escaped = title.replace('"', "\"\"");

    let arg = format!(
        "vbscript:Execute(\"Dim sh, f: Set sh = CreateObject(\"\"Shell.Application\"\"): Set f = sh.BrowseForFolder(0, \"\"{title_escaped}\"\", 0): If Not f Is Nothing Then WScript.StdOut.Write f.self.Path: close\")"
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
    filters: impl IntoIterator<Item = (String, Vec<String>)>,
    filename: String,
    on_picked: impl FnOnce(PathBuf) -> Message + 'static,
) -> Message {
    let title_escaped = title.replace('"', "\"\"");
    let filename_escaped = filename.replace('"', "\"\"");
    let filter_str = get_filter_string(filters);
    let filter_escaped = filter_str.replace('"', "\"\"");

    let arg = format!(
        "vbscript:Execute(\"Dim dlg: Set dlg = CreateObject(\"\"UserAccounts.CommonDialog\"\"): dlg.Title = \"\"{title_escaped}\"\": dlg.FileName = \"\"{filename_escaped}\"\": dlg.Filter = \"\"{filter_escaped}\"\": If dlg.ShowSave Then WScript.StdOut.Write dlg.FileName: close\")"
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
