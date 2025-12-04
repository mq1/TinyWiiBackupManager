// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use egui_file_dialog::{FileDialog, FileDialogLabels};
use std::sync::Arc;

pub fn get_base_dialog() -> FileDialog {
    FileDialog::new()
        .labels(get_labels())
        .as_modal(true)
        .err_icon(egui_phosphor::regular::X)
        .default_file_icon(egui_phosphor::regular::FILE)
        .default_folder_icon(egui_phosphor::regular::FOLDER)
        .device_icon(egui_phosphor::regular::HARD_DRIVE)
        .removable_device_icon(egui_phosphor::regular::HARD_DRIVE)
        .set_file_icon(
            egui_phosphor::regular::DISC,
            Arc::new(|path| path.extension().unwrap_or_default() == "gcm"),
        )
        .set_file_icon(
            egui_phosphor::regular::DISC,
            Arc::new(|path| path.extension().unwrap_or_default() == "iso"),
        )
        .set_file_icon(
            egui_phosphor::regular::DATABASE,
            Arc::new(|path| path.extension().unwrap_or_default() == "wbfs"),
        )
        .set_file_icon(
            egui_phosphor::regular::FILE_ARCHIVE,
            Arc::new(|path| path.extension().unwrap_or_default() == "wia"),
        )
        .set_file_icon(
            egui_phosphor::regular::FILE_ARCHIVE,
            Arc::new(|path| path.extension().unwrap_or_default() == "rvz"),
        )
        .set_file_icon(
            egui_phosphor::regular::FILE_ARCHIVE,
            Arc::new(|path| path.extension().unwrap_or_default() == "ciso"),
        )
        .set_file_icon(
            egui_phosphor::regular::FILE_ARCHIVE,
            Arc::new(|path| path.extension().unwrap_or_default() == "gcz"),
        )
        .set_file_icon(
            egui_phosphor::regular::FILE_ARCHIVE,
            Arc::new(|path| path.extension().unwrap_or_default() == "tgc"),
        )
        .set_file_icon(
            egui_phosphor::regular::FILES,
            Arc::new(|path| path.extension().unwrap_or_default() == "nfs"),
        )
        .set_file_icon(
            egui_phosphor::regular::FILE_ZIP,
            Arc::new(|path| path.extension().unwrap_or_default() == "zip"),
        )
}

fn get_labels() -> FileDialogLabels {
    FileDialogLabels {
        title_select_directory: format!("{} Select Folder", egui_phosphor::regular::FOLDER),
        title_select_file: format!("{} Open File", egui_phosphor::regular::FOLDER_OPEN),
        title_select_multiple: format!("{} Select Multiple", egui_phosphor::regular::FILES),
        title_save_file: format!("{} Save File", egui_phosphor::regular::FILE_ARROW_DOWN),

        cancel: "Cancel".to_string(),
        overwrite: "Overwrite".to_string(),

        reload: format!(
            "{}  Reload",
            egui_phosphor::regular::ARROW_COUNTER_CLOCKWISE
        ),
        working_directory: format!(
            "{}  Go to working directory",
            egui_phosphor::regular::ARROW_UP_RIGHT
        ),
        show_hidden: " Show hidden".to_string(),
        show_system_files: " Show system files".to_string(),

        heading_pinned: "Pinned".to_string(),
        heading_places: "Places".to_string(),
        heading_devices: "Devices".to_string(),
        heading_removable_devices: "Removable Devices".to_string(),

        home_dir: format!("{}  Home", egui_phosphor::regular::HOUSE),
        desktop_dir: format!("{}  Desktop", egui_phosphor::regular::DESKTOP),
        documents_dir: format!("{}  Documents", egui_phosphor::regular::FILES),
        downloads_dir: format!("{}  Downloads", egui_phosphor::regular::FILE_ARROW_DOWN),
        audio_dir: format!("{}  Audio", egui_phosphor::regular::MUSIC_NOTE),
        pictures_dir: format!("{}  Pictures", egui_phosphor::regular::IMAGE),
        videos_dir: format!("{}  Videos", egui_phosphor::regular::VIDEO),

        pin_folder: format!("{} Pin", egui_phosphor::regular::PUSH_PIN),
        unpin_folder: format!("{} Unpin", egui_phosphor::regular::X),
        rename_pinned_folder: format!("{} Rename", egui_phosphor::regular::PENCIL),

        selected_directory: "Selected directory:".to_string(),
        selected_file: "Selected file:".to_string(),
        selected_items: "Selected items:".to_string(),
        file_name: "File name:".to_string(),
        file_filter_all_files: "All Files".to_string(),
        save_extension_any: "Any".to_string(),

        open_button: format!("{}  Open", egui_phosphor::regular::FOLDER_OPEN),
        save_button: format!("{}  Save", egui_phosphor::regular::FILE_ARROW_DOWN),
        cancel_button: format!("{} Cancel", egui_phosphor::regular::X),

        overwrite_file_modal_text: "already exists. Do you want to overwrite it?".to_string(),

        err_empty_folder_name: "Name of the folder cannot be empty".to_string(),
        err_empty_file_name: "The file name cannot be empty".to_string(),
        err_directory_exists: "A directory with the name already exists".to_string(),
        err_file_exists: "A file with the name already exists".to_string(),
    }
}
