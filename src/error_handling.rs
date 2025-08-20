// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use anyhow::Error;
use rfd::{MessageDialog, MessageLevel};
use tracing::error;

/// Show an error dialog with the given title and description
pub fn show_error(title: &str, description: &str) {
    MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title(title)
        .set_description(description)
        .show();
}

/// Show an anyhow::Error in an error dialog
pub fn show_anyhow_error(title: &str, error: &Error) {
    error!("{error:?}");
    let mut description = format!("Error: {error}");
    for cause in error.chain().skip(1) {
        description.push_str(&format!("\nCaused by: {cause}"));
    }
    show_error(title, &description);
}
