// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::path::PathBuf;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree};
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, GetSaveFileNameW, OFN_ALLOWMULTISELECT, OFN_EXPLORER, OFN_FILEMUSTEXIST,
    OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::Shell::{
    BIF_EDITBOX, BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, SHBrowseForFolderW,
    SHGetPathFromIDListW,
};
use windows::core::{PCWSTR, PWSTR};

const MAX_PATH: usize = 260;
const MAX_PATH_LARGE: usize = 32_768;

pub fn get_hwnd(window: &slint::WindowHandle) -> Option<HWND> {
    let handle = handle.window_handle().ok()?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };

    let hwnd = HWND(handle.hwnd.get() as *mut _);

    Some(hwnd)
}

pub fn pick_dir(window: &slint::WindowHandle, title: &str) -> Option<PathBuf> {
    let title_wide = widen(title);
    let mut buf = [0u16; MAX_PATH];

    let hwnd = get_hwnd(window)?;

    let success = unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);

        let mut browse_info = BROWSEINFOW {
            lpszTitle: PCWSTR(title_wide.as_ptr()),
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_EDITBOX | BIF_NEWDIALOGSTYLE,
            hwndOwner: hwnd,
            ..Default::default()
        };

        let pidl = SHBrowseForFolderW(&mut browse_info);
        let success = SHGetPathFromIDListW(pidl as *const _, &mut buf);
        CoTaskMemFree(Some(pidl as *const _));

        success
    };

    if success.as_bool() {
        Some(PathBuf::from(unwiden(&buf)))
    } else {
        None
    }
}

pub fn pick_file(
    window: &slint::WindowHandle,
    title: &str,
    filter: (&str, &[&str]),
) -> Option<PathBuf> {
    let title_wide = widen(title);
    let filter_wide = get_filter_utf16(filter);
    let mut buf = [0u16; MAX_PATH_LARGE];

    let hwnd = get_hwnd(window)?;

    let success = unsafe {
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            lpstrFilter: PCWSTR(filter_wide.as_ptr()),
            lpstrFile: PWSTR(buf.as_mut_ptr()),
            nMaxFile: buf.len() as u32,
            lpstrTitle: PCWSTR(title_wide.as_ptr()),
            hwndOwner: hwnd,
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER,
            ..Default::default()
        };

        GetOpenFileNameW(&mut ofn)
    };

    if success.as_bool() {
        Some(PathBuf::from(unwiden(&buf)))
    } else {
        None
    }
}

pub fn pick_files(
    window: &slint::WindowHandle,
    title: &str,
    filter: (&str, &[&str]),
) -> Vec<PathBuf> {
    let title_wide = widen(title);
    let filter_wide = get_filter_utf16(filter);
    let mut buf = vec![0u16; MAX_PATH_LARGE];

    let Some(hwnd) = get_hwnd(window) else {
        return Vec::new();
    };

    let success = unsafe {
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            lpstrFilter: PCWSTR(filter_wide.as_ptr()),
            lpstrFile: PWSTR(buf.as_mut_ptr()),
            nMaxFile: buf.len() as u32,
            lpstrTitle: PCWSTR(title_wide.as_ptr()),
            hwndOwner: hwnd,
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER | OFN_ALLOWMULTISELECT,
            ..Default::default()
        };

        GetOpenFileNameW(&mut ofn)
    };

    if success.as_bool() {
        parse_multi_select(&buf)
    } else {
        Vec::new()
    }
}

pub fn save_file(
    window: &slint::WindowHandle,
    title: &str,
    filter: (&str, &[&str]),
    filename: &str,
) -> Option<PathBuf> {
    let title_wide = widen(title);
    let filter_wide = get_filter_utf16(filter);
    let filename_wide = widen(filename);

    let mut buf = [0u16; MAX_PATH_LARGE];
    buf[..filename_wide.len()].copy_from_slice(&filename_wide);

    let hwnd = get_hwnd(window)?;

    let success = unsafe {
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            lpstrFilter: PCWSTR(filter_wide.as_ptr()),
            lpstrFile: PWSTR(buf.as_mut_ptr()),
            nMaxFile: buf.len() as u32,
            lpstrTitle: PCWSTR(title_wide.as_ptr()),
            hwndOwner: hwnd,
            Flags: OFN_EXPLORER | OFN_OVERWRITEPROMPT,
            ..Default::default()
        };

        GetSaveFileNameW(&mut ofn)
    };

    if success.as_bool() {
        Some(PathBuf::from(unwiden(&buf)))
    } else {
        None
    }
}

fn widen(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn unwiden(buffer: &[u16]) -> String {
    let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..len])
}

fn get_filter_utf16(filter: (&str, &[&str])) -> Vec<u16> {
    let mut s = String::new();

    s.push_str(&filter.0);
    s.push(char::from(0));

    let extensions = filter
        .1
        .iter()
        .map(|ext| format!("*.{ext}"))
        .collect::<Vec<_>>()
        .join(";");

    s.push_str(&extensions);
    s.push(char::from(0));

    widen(&s)
}

fn parse_multi_select(buffer: &[u16]) -> Vec<PathBuf> {
    let parts: Vec<String> = buffer
        .split(|c| *c == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf16_lossy(s))
        .collect();

    if parts.is_empty() {
        return vec![];
    }

    // Single file selected
    if parts.len() == 1 {
        return vec![PathBuf::from(&parts[0])];
    }

    // Multi-select: first part is directory, rest are filenames
    let dir = PathBuf::from(&parts[0]);
    parts[1..].iter().map(|f| dir.join(f)).collect()
}
