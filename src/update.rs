// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    config::message::ConfigMessage,
    consoles,
    games::{ImportEntry, games_state::GamesState, txtcodes::download_cheats},
    homebrew::homebrew_state::HomebrewState,
    jobs::Job,
    library,
    message::Message,
    notifications::notification::Notification,
    osc::osc_state::OscState,
    state::AppState,
    tasks::{TaskState, WorkerEvent},
    ui::{dialogs, modals::Modal, pages::Page},
};
use iced::Task;
use std::time::Instant;

const NEW_DRIVE_TEXT: &str = "New drive detected (or a breaking app update has been installed)\nA path normalization run is recommended\nYou can find it in the Toolbox page";

/// Messages that write to the selected drive
fn writes_to_drive(message: &Message) -> bool {
    matches!(
        message,
        Message::DeleteDir(_)
            | Message::ImportHomebrewApps(_)
            | Message::RunTool(_)
            | Message::InstallOscApp(_)
            | Message::DownloadTxtCodes(_)
            | Message::PickedGames(_)
            | Message::Scrub(_)
    )
}

impl AppState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        if writes_to_drive(&message)
            && let Some(drive) = self.selected_drive().filter(|drive| drive.read_only())
        {
            self.notifications.push(Notification::error(format!(
                "{} is read-only{}: nothing can be written to it",
                drive.label,
                drive.read_only_reason()
            )));
            return Task::none();
        }

        match message {
            Message::Config(message) => {
                match &message {
                    ConfigMessage::Written(Err(e)) => {
                        self.notifications.push(Notification::error(e));
                    }
                    ConfigMessage::NewMountPoint => {
                        self.notifications.push(Notification::info(NEW_DRIVE_TEXT));
                    }
                    _ => {}
                }

                self.config.update(message).map(Message::Config)
            }
            Message::NoOp => Task::none(),
            Message::Notify(notification) => {
                self.notifications.push(notification);
                Task::none()
            }
            Message::NavigateTo(page) => {
                self.current_page = page;

                if page == Page::Consoles && !self.consoles.searching {
                    return self.update(Message::SearchConsoles);
                }

                if page == Page::Library {
                    let scan = if self.library.scanned_once {
                        Task::none()
                    } else {
                        self.update(Message::ScanLibrary)
                    };
                    return Task::batch([scan, self.load_missing_titles_task()]);
                }

                if page == Page::Osc && matches!(self.osc_contents, OscState::Loaded(_)) {
                    self.reload_all_osc_icons();
                    self.download_osc_icons_task()
                } else {
                    Task::none()
                }
            }
            Message::PickMountPoint => {
                dialogs::init_file_dialog_task().then(dialogs::make_pick_mount_point_dialog_task)
            }
            Message::MountPointPicked(path) => {
                let config_task = self
                    .config
                    .update(ConfigMessage::SetMountPoint(path))
                    .map(Message::Config);

                Task::batch([
                    config_task,
                    self.refresh_games_and_apps(),
                    self.load_drives_task(),
                ])
            }
            Message::CloseNotification(idx) => {
                self.notifications.close(idx);
                Task::none()
            }
            Message::RefreshGamesAndApps => self.refresh_games_and_apps(),
            Message::GotConfig(config) => {
                self.config = config;
                Task::batch([
                    self.refresh_games_and_apps(),
                    self.load_drives_task(),
                    Task::perform(
                        library::load_cache(self.data_dir),
                        Message::GotCachedLibrary,
                    ),
                    self.update(Message::ScanLibrary),
                ])
            }
            Message::GotGames(games) => {
                self.games = games;
                self.reload_all_covers();
                self.download_ui_covers_task()
            }
            Message::GotHomebrew(homebrew) => {
                self.set_homebrew_apps(homebrew);
                Task::none()
            }
            Message::GotDrive(drive) => {
                self.drive = drive;
                Task::none()
            }
            Message::Open(url) => {
                if let Err(e) = open::that(url) {
                    self.notifications.push(Notification::error(e));
                }

                Task::none()
            }
            Message::OpenGameInfo(game) => {
                self.current_modal = Some(Modal::GameInfo((game.clone(), None, false)));

                Task::batch([
                    self.get_disc_info_task(game.clone()),
                    self.check_if_has_update_partition_task(game),
                ])
            }
            Message::OpenHomebrewAppInfo(app) => {
                self.current_modal = Some(Modal::HomebrewAppInfo(app));
                Task::none()
            }
            Message::OpenOscAppInfo(app) => {
                self.current_modal = Some(Modal::OscAppInfo(app));
                Task::none()
            }
            Message::CloseModal => {
                self.current_modal = None;
                Task::none()
            }
            Message::GotDiscInfo(new_meta) => {
                if let Some(Modal::GameInfo((_, meta, _))) = &mut self.current_modal {
                    *meta = Some(new_meta);
                }

                Task::none()
            }
            Message::GotUpdatePartitionCheckResult(has_update_partition) => {
                if let Some(Modal::GameInfo((_, _, scrubbable))) = &mut self.current_modal {
                    *scrubbable = has_update_partition;
                }

                Task::none()
            }
            Message::AskDeleteDir(path) => {
                self.current_modal = Some(Modal::DeleteDir(path));
                Task::none()
            }
            Message::DeleteDir(path) => self.delete_dir_task(path),
            Message::DirDeleted(res) => {
                if let Err(e) = res {
                    self.notifications.push(Notification::error(e));
                }

                Task::done(Message::RefreshGamesAndApps)
            }
            Message::PickHomebrewApps => {
                dialogs::init_file_dialog_task().then(dialogs::make_pick_homebrew_apps_dialog_task)
            }
            Message::ImportHomebrewApps(paths) => self.import_homebrew_apps_task(paths),
            Message::HomebrewAppsImported(n) if n > 0 => {
                self.notifications.push(Notification::success(format!(
                    "{n} Homebrew app(s) successfully imported"
                )));

                Task::batch([self.get_homebrew_apps_task(), self.get_drive_info_task()])
            }
            Message::HomebrewAppsImported(_) => Task::none(),
            Message::CalcGameSha1(game) => {
                self.add_job(Job::Hash {
                    game,
                    mount_point: self.config.mount_point().to_path_buf(),
                });
                self.start_ready_tasks()
            }
            Message::PickGames => {
                if let GamesState::Loaded(games) = &self.games {
                    let existing_ids = games.get_all_game_ids().collect::<Box<[_]>>();

                    dialogs::init_file_dialog_task().then(move |base| {
                        dialogs::make_pick_games_dialog_task(base, existing_ids.clone())
                    })
                } else {
                    Task::none()
                }
            }
            Message::PickGamesRecursively => {
                if let GamesState::Loaded(games) = &self.games {
                    let existing_ids = games.get_all_game_ids().collect::<Box<[_]>>();

                    dialogs::init_file_dialog_task().then(move |base| {
                        dialogs::make_pick_games_recursively_dialog_task(base, existing_ids.clone())
                    })
                } else {
                    Task::none()
                }
            }
            Message::PickedGames(paths) => {
                if paths.is_empty() {
                    self.notifications
                        .push(Notification::info("No new games selected"));
                    Task::none()
                } else {
                    for entry in paths {
                        self.add_job(Job::Import {
                            entry,
                            config: self.config.clone(),
                            drive: self.drive.clone(),
                        });
                    }

                    self.start_ready_tasks()
                }
            }
            Message::TaskEvent(
                id,
                WorkerEvent::Progress {
                    done_bytes,
                    total_bytes,
                    message,
                },
            ) => {
                self.tasks
                    .update_progress(id, done_bytes, total_bytes, message, Instant::now());
                Task::none()
            }
            Message::TaskEvent(id, WorkerEvent::Finished(result)) => {
                self.tasks.finish(id, result);

                if let Some(task) = self.tasks.get(id) {
                    match task.state() {
                        TaskState::Done(message) => {
                            self.notifications.push(Notification::success(message));
                        }
                        TaskState::Failed(reason) => self
                            .notifications
                            .push(Notification::error(format!("{}: {reason}", task.title()))),
                        _ => {}
                    }
                }

                // only the drive shown needs reloading, and only if the job changed its games
                let refresh = match self.jobs.get(&id).and_then(Job::changes_games_on) {
                    Some(mount_point) if mount_point == self.config.mount_point() => {
                        Task::batch([self.get_games_task(), self.get_drive_info_task()])
                    }
                    _ => Task::none(),
                };

                Task::batch([self.start_ready_tasks(), refresh])
            }
            Message::CancelTask(id) => {
                self.tasks.cancel(id);
                self.start_ready_tasks()
            }
            Message::CancelAllTasks => {
                let ids: Vec<_> = self
                    .tasks
                    .iter()
                    .filter(|t| !t.state().is_finished())
                    .map(|t| t.id())
                    .collect();

                for id in ids {
                    self.tasks.cancel(id);
                }

                Task::none()
            }
            Message::PauseTask(id) => {
                self.tasks.pause(id);
                Task::none()
            }
            Message::ResumeTask(id) => {
                self.tasks.resume(id);
                Task::none()
            }
            Message::MoveTask(id, up) => {
                self.tasks.move_waiting(id, up);
                Task::none()
            }
            Message::RetryTask(id) => {
                self.tasks.retry(id);
                self.start_ready_tasks()
            }
            Message::ClearTaskHistory => {
                self.clear_task_history();
                Task::none()
            }
            Message::ToggleAnimationState => {
                if self.tasks.running_count() > 0 {
                    self.animation_state = !self.animation_state;
                }

                Task::none()
            }
            Message::ReloadCover(game_id) => {
                self.reload_cover(game_id);
                Task::none()
            }
            Message::ReloadOscIcon(idx) => {
                self.reload_osc_icon(idx);
                Task::none()
            }
            Message::SearchGames(search_term) => {
                if let GamesState::Loaded(games) = &mut self.games {
                    games.set_search_term(search_term.to_lowercase());
                }

                Task::none()
            }
            Message::SearchHomebrewApps(search_term) => {
                if let HomebrewState::Loaded(apps) = &mut self.homebrew {
                    apps.set_search_term(search_term.to_lowercase());
                }

                Task::none()
            }
            Message::ToggleShowWii(checked) => {
                if let GamesState::Loaded(games) = &mut self.games {
                    games.set_show_wii(checked);
                }

                Task::none()
            }
            Message::ToggleShowNgc(checked) => {
                if let GamesState::Loaded(games) = &mut self.games {
                    games.set_show_ngc(checked);
                }

                Task::none()
            }
            Message::PickExportDest(game) => dialogs::init_file_dialog_task().then(move |base| {
                dialogs::make_pick_export_game_dest_dialog_task(base, game.clone())
            }),
            Message::ExportGame(game, out_path) => {
                self.add_job(Job::Export {
                    game,
                    out_path,
                    mount_point: self.config.mount_point().to_path_buf(),
                });
                self.start_ready_tasks()
            }
            Message::RunTool(tool) => self.run_tool(tool),
            Message::PickFileToSendViaWiiload => dialogs::init_file_dialog_task()
                .then(dialogs::make_pick_file_to_wiiload_dialog_task),
            Message::SendViaWiiload(path) => Task::batch([
                self.config
                    .update(ConfigMessage::Write)
                    .map(Message::Config),
                self.send_via_wiiload(path),
            ]),
            Message::RefreshOscContents => {
                Task::perform(OscState::load(self.data_dir, true), Message::GotOscContents)
            }
            Message::GotOscContents(contents) => {
                self.set_osc_contents(contents);

                if self.current_page == Page::Osc
                    && matches!(self.osc_contents, OscState::Loaded(_))
                {
                    self.reload_all_osc_icons();
                    self.download_osc_icons_task()
                } else {
                    Task::none()
                }
            }
            Message::SearchOscApps(search_term) => {
                if let OscState::Loaded(apps) = &mut self.osc_contents {
                    apps.set_search_term(search_term.to_lowercase());
                }

                Task::none()
            }
            Message::InstallOscApp(app) => {
                self.notifications
                    .push(Notification::info(format!("Installing {}", app.name())));

                let app_name = app.name().to_string();
                let mount_point = self.config.mount_point().to_path_buf();

                Task::perform(
                    async move { app.install(&mount_point).await },
                    move |res| match res {
                        Ok(()) => Message::OscAppInstalled(Ok(app_name)),
                        Err(e) => Message::OscAppInstalled(Err(e.to_string())),
                    },
                )
            }
            Message::OscAppInstalled(res) => {
                let notification = match res {
                    Ok(app_name) => Notification::success(format!("Installed {app_name}")),
                    Err(err) => Notification::error(err),
                };

                self.notifications.push(notification);
                Task::batch([self.get_homebrew_apps_task(), self.get_drive_info_task()])
            }
            Message::DownloadTxtCodes(game) => {
                let game_title = game.title().to_string();
                let config = self.config.clone();

                self.notifications.push(Notification::info(format!(
                    "Downloading cheats for {game_title}",
                )));

                Task::perform(
                    async move { download_cheats(game.id(), &config).await },
                    move |res| match res {
                        Ok(()) => {
                            Notification::success(format!("Downloaded cheats for {game_title}"))
                        }
                        Err(err) => Notification::error(err.to_string()),
                    },
                )
                .map(Message::Notify)
            }
            Message::SendOscAppViaWiiload(app) => {
                let app_name: Box<str> = app.name().into();
                let wii_ip: Box<str> = self.config.wii_ip().into();

                self.notifications.push(Notification::info(format!(
                    "Sending {app_name} via wiiload"
                )));

                Task::batch([
                    self.config
                        .update(ConfigMessage::Write)
                        .map(Message::Config),
                    Task::perform(
                        {
                            let wii_ip = wii_ip.clone();
                            async move { app.wiiload(&wii_ip).await }
                        },
                        move |res| match res {
                            Ok(excluded_files) if excluded_files.is_empty() => {
                                Notification::success(format!("Sent {app_name} to {wii_ip}"))
                            }
                            Ok(excluded_files) => Notification::info(format!(
                                "Sent {app_name} to {wii_ip} ({} excluded files)",
                                excluded_files.len()
                            )),
                            Err(err) => Notification::error(err.to_string()),
                        },
                    )
                    .map(Message::Notify),
                ])
            }
            Message::PickGameToConvert => {
                dialogs::init_file_dialog_task().then(dialogs::make_pick_in_out_dialogs_task)
            }
            Message::ConvertGame(disc_path, out_path) => {
                self.add_job(Job::Convert {
                    disc_path,
                    out_path,
                });
                self.start_ready_tasks()
            }
            Message::GotKnownConsoles(entries) => {
                self.consoles.set_known(entries);
                // so that the library can show what is on them
                self.update(Message::SearchConsoles)
            }
            Message::SearchConsoles => {
                self.consoles.searching = true;
                Task::batch([
                    Task::perform(
                        consoles::search(self.consoles.known_addresses()),
                        Message::GotConsoles,
                    ),
                    self.load_drives_task(),
                ])
            }
            Message::GotDrives(drives) => {
                self.consoles.drives = drives;
                Task::none()
            }
            Message::OpenDrive(path, page) => {
                self.current_page = page;
                if path == self.config.mount_point() {
                    return Task::none();
                }
                self.update(Message::MountPointPicked(path))
            }
            Message::ForgetDrive(path) => Task::batch([
                self.config.forget_drive(&path).map(Message::Config),
                self.refresh_games_and_apps(),
                Task::done(Message::GotDrives(
                    self.consoles
                        .drives
                        .iter()
                        .filter(|drive| drive.path != path)
                        .cloned()
                        .collect(),
                )),
            ]),
            Message::GotConsoles(result) => {
                self.consoles.searching = false;
                self.consoles.searched_once = true;

                match result {
                    Ok(found) => {
                        self.consoles.merge(found);
                        let titles = if self.current_page == Page::Library {
                            self.load_missing_titles_task()
                        } else {
                            Task::none()
                        };
                        let data_dir = self.data_dir;
                        let entries = self.consoles.entries.clone();
                        Task::batch([
                            titles,
                            Task::perform(consoles::save(data_dir, entries), |res| match res {
                                Ok(()) => Message::NoOp,
                                Err(e) => Message::Notify(e.into()),
                            }),
                        ])
                    }
                    Err(e) => Task::done(Message::Notify(Notification::error(e))),
                }
            }
            Message::OpenConsole(addr) => {
                self.consoles.selected = Some(addr);
                self.update(Message::LoadConsoleTitles(addr))
            }
            Message::LoadConsoleTitles(addr) => {
                self.consoles
                    .titles
                    .insert(addr, consoles::TitlesState::Loading);
                Task::perform(consoles::load_titles(self.data_dir, addr), move |res| {
                    Message::GotConsoleTitles(addr, res)
                })
            }
            Message::CloseConsole => {
                self.consoles.selected = None;
                Task::none()
            }
            Message::GotConsoleTitles(addr, result) => {
                let titles = match result {
                    Ok(titles) => consoles::TitlesState::Loaded(titles),
                    Err(e) => consoles::TitlesState::Failed(e),
                };
                self.consoles.titles.insert(addr, titles);
                Task::none()
            }
            Message::ScanLibrary => {
                self.library.scanning = true;
                let sources = self.config.rom_sources().to_vec();
                Task::perform(library::scan_async(sources), Message::GotLibrary)
            }
            Message::GotCachedLibrary(items) => {
                // a finished scan is newer than the cache
                if !self.library.scanned_once {
                    self.library.items = items;
                }
                Task::none()
            }
            Message::GotLibrary(items) => {
                let save = Task::perform(
                    library::save_cache(self.data_dir, items.clone()),
                    |res| match res {
                        Ok(()) => Message::NoOp,
                        Err(e) => Message::Notify(e.into()),
                    },
                );
                self.library.items = items;
                self.library.scanning = false;
                self.library.scanned_once = true;
                if self
                    .library
                    .platform
                    .is_some_and(|p| !self.library.platforms().contains(&p))
                {
                    self.library.platform = None;
                }
                save
            }
            Message::FilterLibrary(platform) => {
                self.library.platform = platform;
                Task::none()
            }
            Message::SearchLibrary(search) => {
                self.library.search = search;
                Task::none()
            }
            Message::ToggleRomSources => {
                self.library.show_sources = !self.library.show_sources;
                Task::none()
            }
            Message::PickRomSource => {
                dialogs::init_file_dialog_task().then(dialogs::make_pick_rom_source_dialog_task)
            }
            Message::AddRomSource(path) => Task::batch([
                self.config.add_rom_source(path).map(Message::Config),
                self.update(Message::ScanLibrary),
            ]),
            Message::RemoveRomSource(path) => Task::batch([
                self.config.remove_rom_source(&path).map(Message::Config),
                self.update(Message::ScanLibrary),
            ]),
            Message::CopyToDrive(path) => {
                if self.config.mount_point().as_os_str().is_empty() {
                    return Task::done(Message::Notify(Notification::warning(
                        "Select a drive in My Consoles first",
                    )));
                }
                Task::perform(ImportEntry::new(path), |res| match res {
                    Ok(entry) => Message::PickedGames(vec![entry]),
                    Err(e) => Message::Notify(e.into()),
                })
            }
            Message::GotUpdate(new_version) => {
                self.new_version = Some(new_version);
                Task::none()
            }
            Message::FileDropped(path) => match self.current_page {
                Page::Games => Task::perform(ImportEntry::new(path), |res| match res {
                    Ok(entry) => Message::PickedGames(vec![entry]),
                    Err(e) => Message::Notify(e.into()),
                }),
                Page::HomebrewApps => Task::done(Message::ImportHomebrewApps(vec![path])),
                _ => Task::none(),
            },
            Message::AskConfirmScrub(game) => {
                self.current_modal = Some(Modal::ConfirmScrub(game));
                Task::none()
            }
            Message::Scrub(game) => {
                self.current_modal = None;
                self.add_job(Job::Scrub {
                    game,
                    mount_point: self.config.mount_point().to_path_buf(),
                });
                self.start_ready_tasks()
            }
        }
    }
}
