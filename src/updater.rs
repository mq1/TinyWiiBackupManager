// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Result;
use rfd::{MessageDialog, MessageDialogResult};
use serde::Deserialize;

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/mq1/TinyWiiBackupManager/releases/latest";
const RELEASES_BASE_URL: &str = "https://github.com/mq1/TinyWiiBackupManager/releases/tag/";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
}

pub async fn check_for_updates() -> Result<()> {
    let latest_release = ureq::get(LATEST_RELEASE_URL)
        .call()?
        .into_json::<Release>()?
        .tag_name;

    if env!("CARGO_PKG_VERSION") != latest_release {
        let result = MessageDialog::new().set_title("Update available").set_description(format!("A new version of TinyWiiBackupManager is available: {}.\nDo you want to download it?", latest_release)).set_buttons(rfd::MessageButtons::YesNo).show();

        if result == MessageDialogResult::Yes {
            open::that(format!("{}{}", RELEASES_BASE_URL, latest_release))?;
        }
    } else {
        let _ = MessageDialog::new()
            .set_title("No updates available")
            .set_description("You are using the latest version of TinyWiiBackupManager.")
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }

    Ok(())
}
