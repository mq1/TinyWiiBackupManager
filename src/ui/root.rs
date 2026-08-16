// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ViewAs,
    messages::Message,
    state::AppState,
    ui::{
        components,
        modals::{self, Modal},
        pages::{self, Page},
    },
};
use iced::{
    Color, Element, Length, Theme, border,
    widget::{Column, container, opaque, row, rule, stack},
};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mut stack = stack![row![
        components::sidebar::view(state),
        container(match (state.current_page, state.config.view_as) {
            (Page::Games, ViewAs::Grid) => pages::game_grid::view(state),
            (Page::Games, ViewAs::Table) => pages::game_table::view(state),
            (Page::HomebrewApps, ViewAs::Grid) => pages::homebrew_app_grid::view(state),
            (Page::HomebrewApps, ViewAs::Table) => pages::homebrew_app_table::view(state),
            (Page::Settings, _) => pages::settings::view(),
            (Page::Toolbox, _) => pages::toolbox::view(state),
            (Page::About, _) => pages::about::view(state),
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
                Modal::GameInfo((game, disc_info)) => {
                    modals::game_info::view(game, disc_info.as_ref())
                }
                Modal::HomebrewAppInfo(app) => modals::homebrew_app_info::view(app),
                Modal::DeleteDir(path) => modals::delete_dir::view(path),
            })
            .center(Length::Fill)
            .style(|theme| container::transparent(theme).background(Color::BLACK.scale_alpha(0.7))),
        ));
    }

    if state.notifications.has_notifications() || !state.status.is_empty() {
        stack = stack.push(
            container(components::notifications::view(state))
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
