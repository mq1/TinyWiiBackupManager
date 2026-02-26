// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use iced::Window;
use iced::window::raw_window_handle::RawWindowHandle;
use std::ffi::c_void;
use std::path::PathBuf;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree};
use windows::Win32::UI::Controls::Dialogs::{
    GetOpenFileNameW, GetSaveFileNameW, OFN_ALLOWMULTISELECT, OFN_EXPLORER, OFN_FILEMUSTEXIST,
    OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
};
use windows::Win32::UI::Shell::{
    BIF_RETURNONLYFSDIRS, BROWSEINFOW, SHBrowseForFolderW, SHGetPathFromIDListW,
};
use windows::core::{PCWSTR, PWSTR};

pub fn pick_dir(window: &dyn Window, title: &str) -> Option<PathBuf> {
    let Ok(handle) = window.window_handle() else {
        return None;
    };

    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };

    // Ensure COM is initialized for this thread before using Shell APIs.
    // Safe to call multiple times (returns S_FALSE if already initialized).
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };

    let hwnd = HWND(handle.hwnd.get() as *mut c_void);
    let title_wide = widen(title);

    let mut browse_info = BROWSEINFOW {
        hwndOwner: hwnd,
        lpszTitle: PCWSTR(title_wide.as_ptr()),
        ulFlags: BIF_RETURNONLYFSDIRS,
        ..Default::default()
    };

    let raw_pidl = unsafe { SHBrowseForFolderW(&mut browse_info) };

    if raw_pidl.is_null() {
        return None;
    }

    let pidl_guard = Pidl(raw_pidl as *mut _);

    let mut pszpath = [0u16; 260];
    let success = unsafe { SHGetPathFromIDListW(pidl_guard.0 as *const _, &mut pszpath) };

    if success.as_bool() {
        Some(PathBuf::from(unwiden(&pszpath)))
    } else {
        None
    }
}

pub fn pick_file(window: &dyn Window, title: &str, filter: (&str, &[&str])) -> Option<PathBuf> {
    let title_wide = widen(title);
    let filter_wide = get_filter_utf16(filter);

    let Ok(handle) = window.window_handle() else {
        return None;
    };

    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };

    let hwnd = HWND(handle.hwnd.get() as *mut c_void);

    let mut file_buffer = vec![0u16; 32_768];

    let result = unsafe {
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: hwnd,
            lpstrFilter: PCWSTR(filter_wide.as_ptr()),
            lpstrFile: PWSTR(file_buffer.as_mut_ptr()),
            nMaxFile: file_buffer.len() as u32,
            lpstrTitle: PCWSTR(title_wide.as_ptr()),
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER,
            ..Default::default()
        };

        GetOpenFileNameW(&mut ofn).as_bool()
    };

    if result {
        Some(PathBuf::from(unwiden(&file_buffer)))
    } else {
        None
    }
}

pub fn pick_files(window: &dyn Window, title: &str, filter: (&str, &[&str])) -> Vec<PathBuf> {
    let title_wide = widen(title);
    let filter_wide = get_filter_utf16(filter);

    let Ok(handle) = window.window_handle() else {
        return Vec::new();
    };

    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return Vec::new();
    };

    let hwnd = HWND(handle.hwnd.get() as *mut c_void);

    let mut file_buffer = vec![0u16; 32_768];

    let result = unsafe {
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: hwnd,
            lpstrFilter: PCWSTR(filter_wide.as_ptr()),
            lpstrFile: PWSTR(file_buffer.as_mut_ptr()),
            nMaxFile: file_buffer.len() as u32,
            lpstrTitle: PCWSTR(title_wide.as_ptr()),
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_EXPLORER | OFN_ALLOWMULTISELECT,
            ..Default::default()
        };

        GetOpenFileNameW(&mut ofn).as_bool()
    };

    if result {
        parse_multi_select(&file_buffer)
    } else {
        Vec::new()
    }
}

pub fn save_file(
    window: &dyn Window,
    title: &str,
    filter: (&str, &[&str]),
    filename: &str,
) -> Option<PathBuf> {
    let title_wide = widen(title);
    let filter_wide = get_filter_utf16(filter);

    let Ok(handle) = window.window_handle() else {
        return None;
    };

    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };

    let hwnd = HWND(handle.hwnd.get() as *mut c_void);

    let mut file_buffer = vec![0u16; 32_768];

    let filename_wide = widen(filename);
    file_buffer[..filename_wide.len()].copy_from_slice(&filename_wide);

    let result = unsafe {
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: hwnd,
            lpstrFilter: PCWSTR(filter_wide.as_ptr()),
            lpstrFile: PWSTR(file_buffer.as_mut_ptr()),
            nMaxFile: file_buffer.len() as u32,
            lpstrTitle: PCWSTR(title_wide.as_ptr()),
            Flags: OFN_EXPLORER | OFN_OVERWRITEPROMPT,
            ..Default::default()
        };

        GetSaveFileNameW(&mut ofn).as_bool()
    };

    if result {
        Some(PathBuf::from(unwiden(&file_buffer)))
    } else {
        None
    }
}

struct Pidl(*mut std::ffi::c_void);
impl Drop for Pidl {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0)) };
    }
}

fn widen<S: AsRef<str>>(s: S) -> Vec<u16> {
    s.as_ref()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect()
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

    widen(s)
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
