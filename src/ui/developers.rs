// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

/// Map of developers to their emojis
/// If you want to add yourself, open a PR or an issue :)
pub fn get_developer_emoji(developer: &str) -> &'static char {
    match developer {
        "blackb0x" => &'💫',
        _ => &'👸',
    }
}
