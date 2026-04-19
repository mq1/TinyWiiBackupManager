// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Rust, model::AppModel};
use slint::WindowHandle;

impl Rust {
    pub fn register_callbacks(&self, state: &AppModel, window: WindowHandle) {
        self.on_open(|uri| open::that(&uri).into());

        let state_clone = state.clone();
        let window_clone = window.clone();
        self.on_pick_mount_point(move || {
            if let Some(path) = dialogs::pick_mount_point(window_clone) {
                state_clone.set_mount_point(path);
            }
        });

        let state_clone = state.clone();
        self.on_set_view_as(move |view_as| {
            state_clone.set_view_as(view_as);
        });

        let state_clone = state.clone();
        self.on_set_sort_by(move |sort_by| {
            state_clone.set_sort_by(sort_by);
        });

        self.on_get_drive_info(|path| DriveInfo::from_path(&path));

        let state_clone = state.clone();
        self.on_delete_dir(|path| {
            if let Err(e) = fs::remove_dir_all(&path) {
                state_clone.add_notification(e.into());
            }
        });

        let state_clone = state.clone();
        self.on_load_games(move |path| {
            let path = Path::new(&path);
            let new = game::scan_drive(path);
            state_clone.set_games(new);
        });

        let state_clone = state.clone();
        self.on_load_homebrew_apps(move |path| {
            let path = Path::new(&path);
            let new = homebrew_app::scan_drive(path).unwrap_or_default();
            state_clone.set_homebrew_apps(new);
        });

        let state_clone = state.clone();
        self.on_load_osc_apps(move |force_refresh| {
            let (new, h, min) = osc::load_contents(force_refresh).unwrap_or_default();
            state_clone.set_osc_apps(new);
            (h, min)
        });

        let state_clone = state.clone();
        self.on_filter_games(move |filter| state_clone.set_games_filter(filter));

        let state_clone = state.clone();
        self.on_filter_homebrew_apps(move |filter| state_clone.set_homebrew_apps_filter(filter));

        let state_clone = state.clone();
        self.on_filter_osc_apps(move |filter| state_clone.set_osc_apps_filter(filter));

        self.on_get_disc_info(|game_dir| DiscInfo::try_from_game_dir(Path::new(&game_dir)).into());

        let weak = app.as_weak();
        self.on_pick_games(move |existing_games| {
            let app = weak.upgrade().unwrap();
            let paths = dialogs::pick_games(app.window());
            let existing_ids = existing_games.iter().map(|g| g.id).collect::<Vec<_>>();
            let queue = standard_conversion::make_queue(paths, &existing_ids);
            let state = VecModel::from(queue);
            ModelRc::from(Rc::new(state))
        });

        let weak = app.as_weak();
        self.on_pick_games_r(move |existing_games| {
            let app = weak.upgrade().unwrap();
            let paths = dialogs::pick_games_r(app.window());
            let existing_ids = existing_games.iter().map(|g| g.id).collect::<Vec<_>>();
            let queue = standard_conversion::make_queue(paths, &existing_ids);
            let state = VecModel::from(queue);
            ModelRc::from(Rc::new(state))
        });

        let state_clone = state.clone();
        self.on_sort(move |sort_by| state_clone.sort(sort_by));

        let state_clone = state.clone();
        self.on_set_show_wii(move |show_wii| state_clone.set_show_wii(show_wii));

        let state_clone = state.clone();
        self.on_set_show_gc(move |show_gc| state_clone.set_show_gc(show_gc));

        let state_clone = state.clone();
        self.on_close_notification(|i| {
            state_clone.close_notification(i);
        });

        let state_clone = state.clone();
        self.on_add_conversions_to_queue(|new| {
            state_clone.add_conversions_to_queue(new);
        });

        let weak = app.as_weak();
        self.on_pick_archive_dest(move |game| {
            let app = weak.unwrap();

            match dialogs::save_game(app.window(), &game) {
                Some(path) => path.to_string_lossy().to_shared_string(),
                None => SharedString::new(),
            }
        });

        let weak = app.global::<State<'_>>().as_weak();
        self.on_run_conversion(move |queue, conf, drive_info| {
            let queue = queue
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap();

            let queued = queue.remove(0);
            let mut conv = Conversion::new(&queued, &conf, &drive_info);

            let weak = weak.clone();
            let _ = std::thread::spawn(move || {
                let res = conv.perform(&weak);

                let _ = weak.upgrade_in_event_loop(move |state| {
                    state.invoke_finished_converting(res.into());
                });
            });
        });

        let _weak = app.global::<State<'_>>().as_weak();
        self.on_load_osc_icons(move |_apps| {
            // TODO
            //osc::load_icons(&apps, weak.clone());
        });

        let weak = app.as_weak();
        self.on_install_homebrew_apps(move |mount_point| {
            let app = weak.upgrade().unwrap();
            let paths = dialogs::pick_homebrew_apps(app.window());
            let mount_point = Path::new(&mount_point);

            let res = || -> Result<usize> {
                let count = paths.len();

                for path in paths {
                    let mut f = File::open(path)?;
                    let mut archive = ZipArchive::new(&mut f)?;
                    archive.extract(mount_point)?;
                }

                Ok(count)
            }();

            res.into()
        });

        // TODO
        #[cfg(false)]
        let weak = app.global::<State<'_>>().as_weak();
        #[cfg(false)]
        self.on_cache_covers(move || {
            let ids = weak
                .upgrade()
                .unwrap()
                .get_game_list()
                .games
                .iter()
                .map(|g| g.id.to_string())
                .collect::<Vec<_>>();

            let weak = weak.clone();
            let _ = std::thread::spawn(move || {
                for game_id in ids {
                    if let Err(e) = covers::cache_cover(&game_id) {
                        eprintln!("ERR: Failed to cache cover for {game_id}: {e}");
                    }

                    let _ = weak.upgrade_in_event_loop(move |state| {
                        let mut game_list = state.get_game_list();
                        game_list.reload_cover(&game_id);
                        state.set_game_list(game_list);
                    });
                }
            });
        });

        let weak = app.global::<State<'_>>().as_weak();
        self.on_checksum(move |game| {
            let game_dir = PathBuf::from(&game.path);
            let is_wii = game.is_wii;
            let game_id = game.id.to_string();

            let weak = weak.clone();
            let _ = std::thread::spawn(move || {
                if let Err(e) = checksum::perform(game_dir, is_wii, &game_id, &weak) {
                    let _ = weak.upgrade_in_event_loop(move |state| {
                        state.invoke_notify_err(e.to_shared_string());
                    });
                }
            });
        });

        #[cfg(windows)]
        {
            let weak = app.as_weak();
            self.on_set_window_color(move |is_dark| {
                let app = weak.upgrade().unwrap();
                window_color::set(app.window(), is_dark);
            });
        }
    }
}
