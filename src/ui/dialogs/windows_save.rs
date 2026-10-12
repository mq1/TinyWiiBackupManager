// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::games::export::ExportFormat;
use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf};
use windows_sys::Win32::UI::Controls::Dialogs::{
    CommDlgExtendedError, GetSaveFileNameW, OFN_EXPLORER, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT,
    OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub(super) fn save_game_file(title: &str, filename: &str) -> Option<PathBuf> {
    // Keep the legacy Windows dialog API so the x86/x64 builds still support XP.
    // Unlike tinyfiledialogs, it exposes a separate entry for every output format.
    let mut filters = Vec::new();
    for format in ExportFormat::ALL {
        let extension = format.extension();
        filters.extend(format!("{format} (*.{extension})\0*.{extension}\0").encode_utf16());
    }
    filters.push(0);

    let title: Vec<_> = title.encode_utf16().chain(Some(0)).collect();
    let default_extension: Vec<_> = ExportFormat::default()
        .extension()
        .encode_utf16()
        .chain(Some(0))
        .collect();
    let filename: Vec<_> = filename.encode_utf16().collect();
    let mut path_buffer = vec![0_u16; 32_768];
    if filename.len() >= path_buffer.len() {
        eprintln!("Cannot open the game save dialog: filename is too long");
        return None;
    }
    path_buffer[..filename.len()].copy_from_slice(&filename);

    // Only own a window from this process: focus may have moved to another app
    // while the blocking dialog task was being scheduled.
    // SAFETY: Both calls only query window handles/process IDs.
    let owner = unsafe {
        let foreground = GetForegroundWindow();
        let mut process = 0;
        GetWindowThreadProcessId(foreground, &mut process);
        if process == std::process::id() {
            foreground
        } else {
            std::ptr::null_mut()
        }
    };
    let mut dialog = OPENFILENAMEW {
        lStructSize: size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFilter: filters.as_ptr(),
        nFilterIndex: 1,
        lpstrFile: path_buffer.as_mut_ptr(),
        nMaxFile: path_buffer.len() as u32,
        lpstrTitle: title.as_ptr(),
        lpstrDefExt: default_extension.as_ptr(),
        Flags: OFN_EXPLORER | OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
        ..Default::default()
    };

    // SAFETY: All input strings are NUL-terminated UTF-16 buffers that stay alive
    // for the synchronous call. The writable buffer has nMaxFile elements.
    if unsafe { GetSaveFileNameW(&mut dialog) } == 0 {
        // SAFETY: This reads the calling thread's last common-dialog error.
        let error = unsafe { CommDlgExtendedError() };
        if error != 0 {
            eprintln!("Cannot open the game save dialog: Windows error {error:#x}");
        }
        return None;
    }

    let length = path_buffer.iter().position(|&unit| unit == 0)?;
    // The dialog applies the selected extension before checking for overwrites.
    // Preserve that exact path so confirmation and the export name agree.
    Some(PathBuf::from(OsString::from_wide(&path_buffer[..length])))
}
