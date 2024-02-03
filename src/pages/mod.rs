// SPDX-FileCopyrightText: 2024 Manuel Quarneti <manuelquarneti@protonmail.com>
// SPDX-License-Identifier: GPL-2.0-only

pub mod adding_games;
pub mod drives;
pub mod games;

pub enum Page {
    Drives,
    Games,
    AddingGames,
}
