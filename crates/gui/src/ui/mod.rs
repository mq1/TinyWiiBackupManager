// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod components;
pub mod developers;
pub mod dialogs;
pub mod modals;
pub mod pages;
pub mod root;
pub mod theme;
pub mod title;

#[cfg(target_os = "windows")]
pub mod window_color;
