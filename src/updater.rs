// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Result;
use rfd::{MessageButtons, MessageDialog, MessageDialogResult};
use serde::Deserialize;

const APP_NAME: &str = "TinyWiiBackupManager";
const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/mq1/TinyWiiBackupManager/releases/latest";
const RELEASES_BASE_URL: &str = "https://github.com/mq1/TinyWiiBackupManager/releases/tag/";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

pub fn _check_for_updates() -> Result<()> {
    let latest_release = ureq::get(LATEST_RELEASE_URL)
        .call()?
        .into_json::<Release>()?
        .tag_name;

    if env!("CARGO_PKG_VERSION") != latest_release {
        let update_message = format!(
            "A new version of {} is available: {}.\nDo you want to download it?",
            APP_NAME, latest_release
        );
        let result = MessageDialog::new()
            .set_title("Update available")
            .set_description(&update_message)
            .set_buttons(MessageButtons::YesNo)
            .show();

        if result == MessageDialogResult::Yes {
            let release_url = format!("{}{}", RELEASES_BASE_URL, latest_release);
            open::that(release_url)?;
        }
    } else {
        let _ = MessageDialog::new()
            .set_title("No updates available")
            .set_description(&format!(
                "You are using the latest version of {}.",
                APP_NAME
            ))
            .set_buttons(MessageButtons::Ok)
            .show();
    }

    Ok(())
}

pub fn check_for_updates() {
    if let Err(err) = _check_for_updates() {
        let _ = MessageDialog::new()
            .set_title("Error")
            .set_description(&format!("Error: {}", err))
            .set_buttons(MessageButtons::Ok)
            .show();
    }
}
