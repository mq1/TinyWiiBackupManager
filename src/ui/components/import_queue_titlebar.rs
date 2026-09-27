// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::conversion_state::ConversionState,
    message::Message,
    state::AppState,
    ui::components::my_button::{MyButtonKind, my_button},
};
use iced::{
    Element,
    widget::{row, space},
};
use lucide_icons::Icon;
use tap::Pipe;

pub fn import_queue_titlebar(state: &AppState) -> Element<'_, Message> {
    row![
        space::horizontal(),
        my_button()
            .label("Start importing")
            .icon(Icon::Play)
            .kind(MyButtonKind::Primary)
            .pipe(|btn| if matches!(state.importing, ConversionState::Idle) {
                btn.on_press(Message::TriggerImport)
            } else {
                btn
            })
    ]
    .padding(10)
    .into()
}
