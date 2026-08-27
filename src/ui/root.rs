// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ViewAs,
    messages::Message,
    state::AppState,
    ui::{
        components::{notifications::notifications, sidebar::sidebar},
        modals::{
            Modal, delete_dir::delete_dir, game_info::game_info,
            homebrew_app_info::homebrew_app_info,
        },
        pages::{
            Page, about::about, game_grid::game_grid, game_table::game_table,
            homebrew_app_grid::homebrew_app_grid, homebrew_app_table::homebrew_app_table,
            import_queue::import_queue, settings::settings, toolbox::toolbox,
        },
    },
};
use iced::{
    Color, Element, Length, Theme, border,
    widget::{Column, container, opaque, row, rule, stack},
};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut stack = stack![row![
        sidebar(state),
        container(match (state.current_page, state.config.view_as) {
            (Page::Games, ViewAs::Grid) => game_grid(state),
            (Page::Games, ViewAs::Table) => game_table(state),
            (Page::HomebrewApps, ViewAs::Grid) => homebrew_app_grid(state),
            (Page::HomebrewApps, ViewAs::Table) => homebrew_app_table(state),
            (Page::Settings, _) => settings(state),
            (Page::Toolbox, _) => toolbox(state),
            (Page::ImportQueue, _) => import_queue(state),
            (Page::About, _) => about(state),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|theme| {
            let mut base = container::bordered_box(theme);
            base.border.radius = border::radius(10);
            base
        })
    ],];

    if let Some(modal) = &state.current_modal {
        stack = stack.push(opaque(
            container(match modal {
                Modal::GameInfo((game, disc_info)) => game_info(game, disc_info.as_ref()),
                Modal::HomebrewAppInfo(app) => homebrew_app_info(app),
                Modal::DeleteDir(path) => delete_dir(path),
            })
            .center(Length::Fill)
            .style(|theme| container::transparent(theme).background(Color::BLACK.scale_alpha(0.7))),
        ));
    }

    if state.notifications.has_notifications() || !state.status.is_empty() {
        stack = stack.push(
            container(notifications(state))
                .align_right(Length::Fill)
                .align_bottom(Length::Fill),
        )
    };

    let mut col = Column::new();

    if cfg!(target_os = "macos") {
        col = col.push(rule::horizontal(32).style(|theme: &Theme| rule::Style {
            color: theme.palette().background,
            ..rule::default(theme)
        }));
    }

    col.push(stack).into()
}
