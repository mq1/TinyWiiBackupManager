// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

pub mod games;
pub mod plugins;
pub mod settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Games,
    Settings,
    Plugins,
}
