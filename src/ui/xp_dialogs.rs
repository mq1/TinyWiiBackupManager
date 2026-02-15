// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{data_dir::get_data_dir, message::Message};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

pub enum Level {
    Info,
    Warning,
    Error,
}

impl Level {
    fn as_str(&self) -> &'static str {
        match self {
            Level::Info => "Info",
            Level::Warning => "Warning",
            Level::Error => "Error",
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

fn alert(title: String, text: String, level: Level) -> Message {
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

fn confirm(
    data_dir: &Path,
    title: &str,
    text: &str,
    level: &Level,
    on_confirm: Message,
) -> Message {
    let script = include_bytes!("../../assets/xp-dialogs/confirm.vbs");

    let vbs_path = data_dir.join("confirm.vbs");
    let vbs_path = match dunce::canonicalize(vbs_path) {
        Ok(path) => path,
        Err(e) => {
            return Message::GenericError(e.to_string());
        }
    };

    let _ = fs::write(&vbs_path, script);

    let res = Command::new("CScript")
        .arg("//Nologo")
        .arg(&vbs_path)
        .arg(title)
        .arg(text)
        .arg(level.as_str())
        .output();

    let output = match res {
        Ok(output) => output,
        Err(e) => {
            return Message::GenericError(e.to_string());
        }
    };

    let output = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output == "yes" {
        on_confirm
    } else {
        Message::None
    }
}

fn pick_file(
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

fn pick_files(
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

fn pick_dir(title: String, on_picked: impl FnOnce(PathBuf) -> Message + 'static) -> Message {
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

fn save_file(
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

pub fn confirm_strip_all_games(data_dir: &Path) -> Message {
    const TITLE: &str = "Remove update partitions?";
    const TEXT: &str = "Are you sure you want to remove the update partitions from all .wbfs files?\n\nThis is irreversible!";
    const LEVEL: &Level = &Level::Warning;
    let on_confirm = Message::StripAllGames;

    confirm(data_dir, TITLE, TEXT, LEVEL, on_confirm)
}
