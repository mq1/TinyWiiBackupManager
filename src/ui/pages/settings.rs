// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{messages::Message, state::AppState};
use iced::{
    Element,
    widget::{column, text},
};

pub fn settings(_state: &AppState) -> Element<'_, Message> {
    column![text("Settings")].padding(10).into()
}
