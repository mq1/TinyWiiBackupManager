// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

// Enable Clippy lints for better code quality
#![warn(clippy::all, rust_2018_idioms)]

// Declare modules
mod app;
mod components;
mod error_handling;
mod game;
mod titles;
mod version_check;

// Re-export the main App struct for use in main.rs
pub use app::App;