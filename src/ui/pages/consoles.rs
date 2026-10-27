// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    consoles::{ConsoleEntry, Drive, InstalledTitle, TitlesState, VCINJECT_LINK_VERSION},
    link::protocol::{Console, n3ds},
    message::Message,
    state::AppState,
    ui::{
        components::{
            my_button::{MyButtonKind, my_button},
            my_card::my_card,
            my_group::my_group,
        },
        pages::Page,
    },
};
use iced::{
    Alignment, Element, Length, padding,
    widget::{Column, column, image, row, scrollable, space, text, tooltip},
};
use lucide_icons::{Icon, iced::icon_alert_triangle};
use size::Size;

pub fn consoles(state: &AppState) -> Element<'_, Message> {
    let consoles = &state.consoles;

    if let Some(addr) = consoles.selected
        && let Some(entry) = consoles.entries.iter().find(|entry| entry.addr == addr)
    {
        return console_titles(entry, consoles.titles.get(&addr));
    }

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

    let network: Element<'_, Message> = if consoles.entries.is_empty() {
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
        .into()
    } else {
        consoles
            .entries
            .iter()
            .map(console_card)
            .collect::<Column<'_, _>>()
            .spacing(10)
            .into()
    };

    let drives: Element<'_, Message> = if consoles.drives.is_empty() {
        text("Add the USB disk or SD card of your Wii or GameCube to manage its games and apps.")
            .size(12)
            .into()
    } else {
        consoles
            .drives
            .iter()
            .map(|drive| drive_card(drive, drive.path == state.config.mount_point()))
            .collect::<Column<'_, _>>()
            .spacing(10)
            .into()
    };

    let drives_header = row![
        text("Drives").size(16),
        space::horizontal(),
        my_button::<Message>()
            .icon(Icon::Plus)
            .label("Add drive")
            .on_press(Message::PickMountPoint),
    ]
    .align_y(Alignment::Center);

    let body = scrollable(
        column![
            text("Network").size(16),
            network,
            space().height(10),
            drives_header,
            drives
        ]
        .spacing(10)
        .padding(padding::all(10).right(20))
        .width(Length::Fill),
    );

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

    let titles_button = entry.online.then(|| {
        my_button()
            .icon(Icon::LayoutGrid)
            .label("Titles")
            .on_press(Message::OpenConsole(entry.addr))
    });

    let content = column![text(app), system, battery, storage, outdated, titles_button].spacing(5);

    my_group(&entry.label, content)
        .icon(if entry.online {
            Icon::Wifi
        } else {
            Icon::WifiOff
        })
        .top_right_content(text(if entry.online { "Online" } else { "Offline" }).size(12))
        .into()
}

fn console_titles<'a>(
    entry: &'a ConsoleEntry,
    titles: Option<&'a TitlesState>,
) -> Element<'a, Message> {
    let titlebar = row![
        my_button::<Message>()
            .icon(Icon::ArrowLeft)
            .kind(MyButtonKind::Toolbar)
            .on_press(Message::CloseConsole),
        text(&entry.label).size(18),
        space::horizontal(),
        tooltip(
            my_button::<Message>()
                .icon(Icon::RefreshCw)
                .kind(MyButtonKind::Toolbar)
                .on_press(Message::OpenConsole(entry.addr)),
            my_card("Reload"),
            tooltip::Position::Bottom
        ),
    ]
    .spacing(10)
    .padding(10)
    .align_y(Alignment::Center);

    let body: Element<'_, Message> = match titles.unwrap_or(&TitlesState::NotLoaded) {
        TitlesState::NotLoaded | TitlesState::Loading => text("Reading the titles…").into(),
        TitlesState::Failed(e) => text(e).into(),
        TitlesState::Loaded(titles) if titles.is_empty() => text("No games or apps.").into(),
        TitlesState::Loaded(titles) => scrollable(
            titles
                .iter()
                .map(title_row)
                .collect::<Column<'_, _>>()
                .spacing(8)
                .padding(padding::all(10).right(20))
                .width(Length::Fill),
        )
        .into(),
    };

    column![titlebar, column![body].padding(padding::horizontal(10))].into()
}

fn title_row(title: &InstalledTitle) -> Element<'_, Message> {
    let icon: Element<'_, Message> = match &title.icon {
        Some(handle) => image(handle).width(48).height(48).into(),
        None => space().width(48).height(48).into(),
    };

    let mut details = Vec::new();
    if !title.publisher.is_empty() {
        details.push(title.publisher.clone());
    }
    if !title.title.product_code.is_empty() {
        details.push(title.title.product_code.clone());
    }
    details.push(Size::from_bytes(title.title.size).to_string());
    if title.title.version > 0 {
        details.push(format!("v{}", title.title.version));
    }

    row![
        icon,
        column![text(&title.name), text(details.join(" · ")).size(12)].spacing(2)
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn drive_card(drive: &Drive, selected: bool) -> Element<'_, Message> {
    let path = text(drive.path.display().to_string()).size(12);

    let space = drive.space.map(|space| {
        text(format!(
            "{} free of {}",
            Size::from_bytes(space.free),
            Size::from_bytes(space.total)
        ))
    });

    let warning = drive.fs_warning().map(|warning| {
        row![
            icon_alert_triangle(),
            text(format!("File system: {warning}"))
        ]
        .spacing(5)
        .align_y(Alignment::Center)
    });

    let online = drive.space.is_some();
    let actions = row![
        online.then(|| {
            my_button()
                .icon(Icon::Gamepad2)
                .label("Games")
                .on_press(Message::OpenDrive(drive.path.clone(), Page::Games))
        }),
        online.then(|| {
            my_button()
                .icon(Icon::Bubbles)
                .label("Homebrew")
                .on_press(Message::OpenDrive(drive.path.clone(), Page::HomebrewApps))
        }),
        space::horizontal(),
        my_button()
            .icon(Icon::Trash)
            .label("Forget")
            .kind(MyButtonKind::Danger)
            .on_press(Message::ForgetDrive(drive.path.clone())),
    ]
    .spacing(5);

    let status = match (online, selected) {
        (false, _) => "Not connected",
        (true, true) => "Connected · selected",
        (true, false) => "Connected",
    };

    my_group(
        &drive.label,
        column![path, space, warning, actions].spacing(5),
    )
    .icon(Icon::HardDrive)
    .top_right_content(text(status).size(12))
    .into()
}
