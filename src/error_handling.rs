// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use rfd::{MessageDialog, MessageLevel};

/// Shows a standardized error dialog.
pub fn show_error(title: &str, description: &str) {
    MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title(title)
        .set_description(description)
        .show();
}
