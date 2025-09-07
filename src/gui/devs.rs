// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use phf::phf_map;

/// Map of developers to their emojis
/// If you want to add yourself, open a PR or an issue :)
pub static DEVELOPERS: phf::Map<&'static str, char> = phf_map! {
    "blackb0x" => '💫'
};
