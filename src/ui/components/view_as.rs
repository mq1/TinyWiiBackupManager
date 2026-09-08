// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ViewAs,
    messages::Message,
    state::AppState,
    ui::components::dual_toggle::{DualToggleItem, dual_toggle},
};
use iced::Element;
use lucide_icons::Icon;

const VIEW_AS_GRID: DualToggleItem<'static> = DualToggleItem {
    icon: Icon::Grid,
    desc: "View as grid",
    active: false,
    on_press: Message::SetViewAs(ViewAs::Grid),
};

const VIEW_AS_TABLE: DualToggleItem<'static> = DualToggleItem {
    icon: Icon::Rows3,
    desc: "View as table",
    active: false,
    on_press: Message::SetViewAs(ViewAs::Table),
};

pub fn view_as(state: &AppState) -> Element<'_, Message> {
    dual_toggle([
        VIEW_AS_GRID.active(state.config.view_as == ViewAs::Grid),
        VIEW_AS_TABLE.active(state.config.view_as == ViewAs::Table),
    ])
}
