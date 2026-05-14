// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::game::Game;
use derive_more::Display;
use std::path::PathBuf;

#[derive(Debug, Clone, Display)]
pub enum QueuedConversion {
    #[display("↑ Conversion: {}", _0.display())]
    Standard(PathBuf),

    #[display("↓ Archive: {}", _1.display())]
    Archive(PathBuf, PathBuf),

    #[display("↔ Scrub: {}", &_0.title)]
    Scrub(Game),
}
