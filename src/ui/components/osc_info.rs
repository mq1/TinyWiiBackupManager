// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{message::Message, state::State};
use iced::{Element, widget::text};

pub fn view(state: &State, osc_i: usize) -> Element<'_, Message> {
    let app = &state.osc_apps[osc_i];

    text(&app.meta.name).into()
}
