// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::ui::settings::settings,
    message::Message,
    state::AppState,
    ui::{
        components::{
            drive_tabs::drive_tabs, notifications::notifications, sidebar::sidebar,
            task_bar::task_bar,
        },
        modals::{
            Modal, confirm_scub::confirm_scrub, delete_dir::delete_dir, game_info::game_info,
            homebrew_app_info::homebrew_app_info, osc_app_info::osc_app_info,
        },
        pages::{
            Page, about::about, consoles::consoles, games::games, homebrew_apps::homebrew_apps,
            library::library, osc_apps::osc_apps, tasks::tasks, toolbox::toolbox,
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
            column![
                container(match state.current_page {
                    Page::Library => library(state),
                    Page::Games => column![drive_tabs(state), games(state)].into(),
                    Page::Consoles => consoles(state),
                    Page::HomebrewApps => column![drive_tabs(state), homebrew_apps(state)].into(),
                    Page::Osc => osc_apps(state),
                    Page::Settings => settings(&state.config).map(Message::Config),
                    Page::Toolbox => toolbox(state),
                    Page::Tasks => tasks(state),
                    Page::About => about(state),
                })
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    border: container::bordered_box(theme).border.rounded(10),
                    ..container::bordered_box(theme)
                }),
                task_bar(state),
            ]
        ],
        state.current_modal.as_ref().map(|modal| {
            opaque(
                container(match modal {
                    Modal::GameInfo((game, disc_info, scrubbable)) => {
                        game_info(game, disc_info.as_ref(), *scrubbable, state)
                    }
                    Modal::HomebrewAppInfo(app) => homebrew_app_info(app),
                    Modal::OscAppInfo(app) => osc_app_info(app),
                    Modal::DeleteDir(path) => delete_dir(path),
                    Modal::ConfirmScrub(game) => confirm_scrub(game),
                })
                .center(Length::Fill)
                .style(|theme: &Theme| container::Style {
                    background: Some(Background::Color(color!(0, 0, 0, 0.7))), // semi-transparent black
                    ..container::transparent(theme)
                }),
            )
        }),
        (state.notifications.has_notifications() || state.long_operations.is_any_running()).then(
            || {
                container(notifications(state))
                    .align_right(Length::Fill)
                    .align_bottom(Length::Fill)
            }
        )
    ];

    // fill title bar
    #[cfg(target_os = "macos")]
    let content = iced::widget::column![
        iced::widget::rule::horizontal(32).style(|theme: &Theme| iced::widget::rule::Style {
            color: theme.palette().background,
            ..iced::widget::rule::default(theme)
        }),
        content
    ];

    content.into()
}
