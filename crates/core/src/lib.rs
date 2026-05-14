// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![warn(clippy::all, rust_2018_idioms)]

pub mod archive;
pub mod banners;
pub mod checksum;
pub mod config;
pub mod conversion_queue;
pub mod convert;
pub mod covers;
pub mod data_dir;
pub mod disc_info;
pub mod drive_info;
pub mod game;
pub mod game_id;
pub mod homebrew_app;
pub mod id_map;
pub mod normalize_dir_layout;
pub mod osc;
pub mod scrub;
pub mod txtcodes;
pub mod updates;
pub mod util;
pub mod wiiload;
