// SPDX-FileCopyrightText: 2026 Yanik Fuchs <yanik.fuchs@me.com>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    library::{ItemId, ItemKind, LibraryItem, Platform},
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
    widget::{
        Column, Row, column, container, image, row, scrollable, space, text, text_input, tooltip,
    },
};
use lucide_icons::Icon;
use size::Size;

pub fn library(state: &AppState) -> Element<'_, Message> {
    let library = &state.library;

    let titlebar = row![
        text_input("Search", &library.search)
            .on_input(Message::SearchLibrary)
            .width(200),
        space::horizontal(),
        library.scanning.then(|| text("Scanning…")),
        library
            .shown_drive_games()
            .any(|item| !item.in_cloud && presence(item, state).is_empty())
            .then(|| tooltip(
                my_button::<Message>()
                    .icon(Icon::HardDriveUpload)
                    .label("Copy all to drive")
                    .on_press(Message::CopyShownToDrive),
                my_card("Copy the Wii and GameCube games shown here that aren't on the drive yet"),
                tooltip::Position::Bottom
            )),
        tooltip(
            my_button::<Message>()
                .icon(Icon::RefreshCw)
                .kind(MyButtonKind::Toolbar)
                .on_press(Message::ScanLibrary),
            my_card("Scan the sources again"),
            tooltip::Position::Bottom
        ),
        tooltip(
            my_button::<Message>()
                .icon(Icon::FolderOpen)
                .kind(MyButtonKind::Toolbar)
                .on_press(Message::ToggleRomSources),
            my_card("Sources: the folders with your games"),
            tooltip::Position::Bottom
        ),
    ]
    .spacing(5)
    .padding(10)
    .align_y(Alignment::Center);

    let chips = Row::with_children(
        std::iter::once(chip("All", library.platform.is_none(), None)).chain(
            library
                .platforms()
                .into_iter()
                .map(|p| chip(p.label(), library.platform == Some(p), Some(p))),
        ),
    )
    .spacing(5)
    .padding(padding::horizontal(10))
    .wrap();

    let sources =
        (library.show_sources || state.config.rom_sources().is_empty()).then(|| sources(state));

    let body: Element<'_, Message> = if state.config.rom_sources().is_empty() {
        space().into()
    } else if library.items.is_empty() {
        text(if library.scanning {
            "Scanning the sources…"
        } else {
            "No games found in the sources."
        })
        .into()
    } else {
        library
            .visible()
            .map(|item| item_row(item, state))
            .collect::<Column<'_, _>>()
            .spacing(8)
            .into()
    };

    column![
        titlebar,
        chips,
        scrollable(
            column![sources, body]
                .spacing(10)
                .padding(padding::all(10).right(20))
                .width(Length::Fill)
        )
    ]
    .spacing(5)
    .into()
}

fn chip(
    label: &'static str,
    active: bool,
    platform: Option<Platform>,
) -> Element<'static, Message> {
    my_button::<Message>()
        .label(label)
        .kind(if active {
            MyButtonKind::Primary
        } else {
            MyButtonKind::Secondary
        })
        .on_press(Message::FilterLibrary(platform))
        .into()
}

fn sources(state: &AppState) -> Element<'_, Message> {
    let list = state
        .config
        .rom_sources()
        .iter()
        .map(|source| {
            row![
                text(source.display().to_string()).width(Length::Fill),
                my_button::<Message>()
                    .icon(Icon::Trash)
                    .kind(MyButtonKind::Danger)
                    .on_press(Message::RemoveRomSource(source.clone())),
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
        })
        .collect::<Column<'_, _>>()
        .spacing(5);

    let hint = state.config.rom_sources().is_empty().then(|| {
        text(
            "Add the folders where your games, ROMs, CIAs and WADs are. They are scanned \
             by file name; files that are only in the cloud are not downloaded.",
        )
        .size(12)
    });

    my_group(
        "Sources",
        column![
            hint,
            list,
            my_button::<Message>()
                .icon(Icon::FolderPlus)
                .label("Add folder")
                .on_press(Message::PickRomSource)
        ]
        .spacing(10),
    )
    .icon(Icon::FolderOpen)
    .into()
}

fn placeholder_icon(platform: Platform) -> Icon {
    match platform {
        Platform::N3ds => Icon::Gamepad2,
        Platform::Wii | Platform::GameCube | Platform::WiiU => Icon::Disc3,
        _ => Icon::Joystick,
    }
}

/// Where an item is already: the online consoles and the selected drive
fn presence(item: &LibraryItem, state: &AppState) -> Vec<String> {
    match &item.id {
        Some(ItemId::Title(id)) => state
            .consoles
            .having_title(*id)
            .map(|entry| entry.label.clone())
            .collect(),
        Some(ItemId::Game(id)) => match &state.games {
            crate::games::games_state::GamesState::Loaded(games)
                if games.get_all_game_ids().any(|g| g.as_str() == id) =>
            {
                let drive = state.config.mount_point();
                vec![
                    drive
                        .file_name()
                        .unwrap_or(drive.as_os_str())
                        .to_string_lossy()
                        .into_owned(),
                ]
            }
            _ => Vec::new(),
        },
        None => Vec::new(),
    }
}

fn item_row<'a>(item: &'a LibraryItem, state: &'a AppState) -> Element<'a, Message> {
    let icon: Element<'_, Message> = match &item.icon {
        Some(handle) => image(handle).width(48).height(48).into(),
        None => container(
            if item.kind == ItemKind::SdBackup {
                Icon::MemoryStick
            } else {
                placeholder_icon(item.platform)
            }
            .widget()
            .size(28),
        )
        .center(48)
        .into(),
    };

    let mut details = vec![item.platform.label().to_string()];
    details.push(
        match item.kind {
            ItemKind::Cia => "CIA",
            ItemKind::Cart => "game card",
            ItemKind::Disc => "disc",
            ItemKind::Wad => "WAD",
            ItemKind::WiiUTitle => "title",
            ItemKind::Rom => "ROM",
            ItemKind::SdBackup => "SD card backup",
            ItemKind::Archive => "archive",
        }
        .to_string(),
    );
    details.push(Size::from_bytes(item.size).to_string());
    if item.in_cloud {
        details.push("in the cloud".to_string());
    }
    if !item.copies.is_empty() {
        details.push(format!("{} copies", item.copies.len() + 1));
    }

    let on = presence(item, state);
    let status: Element<'_, Message> = if on.is_empty() {
        if item.is_drive_game() {
            my_button::<Message>()
                .icon(Icon::HardDriveUpload)
                .label("Copy to drive")
                .on_press(Message::CopyToDrive(item.path.clone()))
                .into()
        } else {
            space().into()
        }
    } else {
        text(format!("✓ {}", on.join(", "))).size(12).into()
    };

    row![
        icon,
        column![text(&item.name), text(details.join(" · ")).size(12)]
            .spacing(2)
            .width(Length::Fill),
        status
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}
