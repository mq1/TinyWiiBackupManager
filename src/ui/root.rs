// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    games::conversion_state::ConversionState,
    messages::Message,
    state::AppState,
    ui::{
        components::{notifications::notifications, sidebar::sidebar},
        modals::{
            Modal, delete_dir::delete_dir, game_info::game_info,
            homebrew_app_info::homebrew_app_info,
        },
        pages::{
            Page, about::about, games::games, homebrew_apps::homebrew_apps,
            import_queue::import_queue, osc_apps::osc_apps, settings::settings, toolbox::toolbox,
        },
    },
};
use iced::{
    Background, Element, Length, Theme, color,
    widget::{column, container, opaque, row, stack},
};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let content = stack![
        row![
            sidebar(state),
            container(match state.current_page {
                Page::Games => games(state),
                Page::HomebrewApps => homebrew_apps(state),
                Page::Osc => osc_apps(state),
                Page::Settings => settings(state),
                Page::Toolbox => toolbox(state),
                Page::ImportQueue => import_queue(state),
                Page::About => about(state),
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|theme: &Theme| container::Style {
                border: container::bordered_box(theme).border.rounded(10),
                ..container::bordered_box(theme)
            })
        ],
        state.current_modal.as_ref().map(|modal| {
            opaque(
                container(match modal {
                    Modal::GameInfo((game, disc_info)) => game_info(game, disc_info.as_ref()),
                    Modal::HomebrewAppInfo(app) => homebrew_app_info(app),
                    Modal::DeleteDir(path) => delete_dir(path),
                })
                .center(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(color!(0, 0, 0, 0.7))), // semi-transparent black
                    ..container::transparent(theme)
                }),
            )
        }),
        (state.notifications.has_notifications()
            || [&state.importing, &state.exporting, &state.hashing]
                .into_iter()
                .any(|conversion| matches!(conversion, ConversionState::Progress(_))))
        .then(|| {
            container(notifications(state))
                .align_right(Length::Fill)
                .align_bottom(Length::Fill)
        })
    ];

    // fill title bar
    #[cfg(target_os = "macos")]
    let content = column![
        iced::widget::rule::horizontal(32).style(|theme: &Theme| iced::widget::rule::Style {
            color: theme.palette().background,
            ..iced::widget::rule::default(theme)
        }),
        content
    ];

    content.into()
}
