// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

use const_format::{Case, concatcp, map_ascii_case, str_split};
use const_str::join;

pub const PRODUCT_NAME: &str = "TinyWiiBackupManager";

pub mod app;
mod base_dir;
mod components;
mod game;
mod messages;
mod settings;
mod task;
mod update_check;
mod wiitdb;

// Only define the lowercase extensions
const SUPPORTED_INPUT_EXTENSIONS_LOWER: &[&str] = &[
    "gcm", "iso", "wbfs", "wia", "rvz", "ciso", "gcz", "tgc", "nfs",
];

// 1. Join the lowercase extensions into a single comma-separated string
const LOWER_STR: &str = join!(SUPPORTED_INPUT_EXTENSIONS_LOWER, ",");

// 2. Create the uppercase version of that string
const UPPER_STR: &str = map_ascii_case!(Case::Upper, LOWER_STR);

// 3. Join the lower and upper strings together
const ALL_EXTENSIONS: &str = concatcp!(LOWER_STR, ",", UPPER_STR);

// 4. Split the string into an array of strings
pub const SUPPORTED_INPUT_EXTENSIONS: &[&str] = &str_split!(ALL_EXTENSIONS, ",");
