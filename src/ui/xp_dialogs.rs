// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![allow(clippy::needless_pass_by_value)]

use crate::{data_dir::get_data_dir, message::Message};
use iced::Window;
use std::{fs, path::PathBuf, process::Command};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum MessageLevel {
    Info,
    Warning,
    Error,
}

impl MessageLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageLevel::Info => "Info",
            MessageLevel::Warning => "Warning",
            MessageLevel::Error => "Error",
        }
    }
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
    let data_dir = match get_data_dir() {
        Ok(dir) => dir,
        Err(_) => std::env::temp_dir(),
    };

    let vbs_path = data_dir.join("alert.vbs");
    let _ = fs::write(
        &vbs_path,
        include_bytes!("../../assets/xp-dialogs/alert.vbs"),
    );

    let _ = Command::new("cscript")
        .arg("//NoLogo")
        .arg(&vbs_path)
        .arg(&title)
        .arg(&text)
        .arg(level.as_str())
        .status();

    Message::None
}

pub fn confirm(
    _: &dyn Window,
    title: String,
    text: String,
    level: MessageLevel,
    on_confirm: Message,
) -> Message {
    let data_dir = match get_data_dir() {
        Ok(dir) => dir,
        Err(_) => std::env::temp_dir(),
    };

    let vbs_path = data_dir.join("confirm.vbs");
    let _ = fs::write(
        &vbs_path,
        include_bytes!("../../assets/xp-dialogs/confirm.vbs"),
    );

    let output = Command::new("cscript")
        .arg("//NoLogo")
        .arg(&vbs_path)
        .arg(&title)
        .arg(&text)
        .arg(level.as_str())
        .output();

    let Ok(output) = output else {
        return Message::None;
    };

    let output = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output == "yes" {
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
