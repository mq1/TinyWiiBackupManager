// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    consoles::{ConsoleEntry, VCINJECT_LINK_VERSION},
    link::protocol::{Console, n3ds},
    message::Message,
    state::AppState,
    ui::components::{
        my_button::{MyButtonKind, my_button},
        my_card::my_card,
        my_group::my_group,
    },
};
use iced::{
    Alignment, Element, Length, padding,
    widget::{Column, column, row, scrollable, space, text, tooltip},
};
use lucide_icons::{Icon, iced::icon_alert_triangle};
use size::Size;

pub fn consoles(state: &AppState) -> Element<'_, Message> {
    let consoles = &state.consoles;

    let titlebar = row![
        text("My Consoles").size(18),
        space::horizontal(),
        consoles.searching.then(|| text("Searching…")),
        tooltip(
            search_button(consoles.searching),
            my_card("Search the network"),
            tooltip::Position::Bottom
        ),
    ]
    .spacing(10)
    .padding(10)
    .align_y(Alignment::Center);

    let body: Element<'_, Message> = if consoles.entries.is_empty() {
        column![
            text(if consoles.searching || !consoles.searched_once {
                "Looking for consoles in your network…"
            } else {
                "No console found."
            }),
            text(
                "Open vcinject Link on your 3DS, in the same Wi-Fi as this computer. \
                 Wii and Wii U companion apps are coming later."
            )
            .size(12),
        ]
        .spacing(5)
        .padding(10)
        .into()
    } else {
        scrollable(
            consoles
                .entries
                .iter()
                .map(console_card)
                .collect::<Column<'_, _>>()
                .spacing(10)
                .padding(padding::all(10).right(20))
                .width(Length::Fill),
        )
        .into()
    };

    column![titlebar, body].into()
}

fn search_button<'a>(searching: bool) -> Element<'a, Message> {
    let button = my_button::<Message>()
        .icon(Icon::RefreshCw)
        .kind(MyButtonKind::Toolbar);

    if searching {
        button.into()
    } else {
        button.on_press(Message::SearchConsoles).into()
    }
}

fn console_card(entry: &ConsoleEntry) -> Element<'_, Message> {
    let info = &entry.info;

    let app = format!(
        "{} {} · {}",
        if info.app().is_empty() {
            "Companion app"
        } else {
            info.app()
        },
        info.version(),
        entry.addr.ip()
    );

    let system = info.get("system").map(|system| {
        let mut line = format!("System {system}");
        if let Some(luma) = info.get("luma").filter(|luma| !luma.is_empty()) {
            line.push_str(&format!(" · Luma3DS {luma}"));
        }
        text(line)
    });

    let battery = info.battery().map(|level| {
        text(format!(
            "Battery {level}/5{}",
            if info.charging() { ", charging" } else { "" }
        ))
    });

    let storage_location = match info.console() {
        Some(Console::N3ds) => n3ds::LOCATION_SD,
        _ => 0,
    };
    let storage = info.storage(storage_location).map(|(free, total)| {
        text(format!(
            "{} free of {}",
            Size::from_bytes(free),
            Size::from_bytes(total)
        ))
    });

    let outdated = entry.outdated().then(|| {
        row![
            icon_alert_triangle(),
            text(format!(
                "vcinject Link is older than version {VCINJECT_LINK_VERSION}: update it"
            ))
        ]
        .spacing(5)
        .align_y(Alignment::Center)
    });

    let content = column![text(app), system, battery, storage, outdated].spacing(5);

    my_group(&entry.label, content)
        .icon(if entry.online {
            Icon::Wifi
        } else {
            Icon::WifiOff
        })
        .top_right_content(text(if entry.online { "Online" } else { "Offline" }).size(12))
        .into()
}
