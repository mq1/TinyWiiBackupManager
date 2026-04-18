// SPDX-FileCopyrightText: 2026 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-3.0-only

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod archive;
mod checksum;
mod config;
mod convert;
mod covers;
mod data_dir;
mod dialogs;
mod disc_info;
mod drive_info;
mod extensions;
mod game;
mod homebrew_app;
mod homebrew_app_meta;
mod id_map;
mod model;
mod notification;
mod osc;
mod results;
mod scrub;
mod standard_conversion;
mod util;

#[cfg(windows)]
mod window_color;

#[cfg(windows)]
mod xp_dialogs;

use crate::{convert::Conversion, data_dir::DATA_DIR, model::AppModel};
use anyhow::{Result, bail};
use slint::{ComponentHandle, Model, ModelRc, SharedString, ToSharedString, VecModel};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
};
use zip::ZipArchive;

slint::include_modules!();

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn restart_with_sw_rendering() -> Result<()> {
    let exe = std::env::current_exe()?;

    let mut cmd = Command::new(exe);
    cmd.env("SLINT_BACKEND", "winit-software");

    let _ = cmd.spawn()?;

    std::process::exit(0);
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    if DATA_DIR.as_os_str().is_empty() {
        bail!("Failed to get data dir");
    }

    let config = Config::load();

    let model = AppModel::new(
        config.contents.sort_by,
        config.contents.show_wii,
        config.contents.show_gc,
    );
    let app = AppWindow::new()?;

    app.global::<State<'_>>().set_config(config);
    model.init_state(&app.global());

    app.global::<State<'_>>()
        .set_version(env!("CARGO_PKG_VERSION").into());

    app.global::<State<'_>>()
        .set_data_dir(DATA_DIR.to_string_lossy().to_shared_string());

    app.global::<Rust<'_>>().on_load_config(Config::load);

    app.global::<Rust<'_>>()
        .on_open(|uri| open::that(&uri).into());

    let weak = app.as_weak();
    app.global::<Rust<'_>>().on_pick_mount_point(move || {
        let app = weak.upgrade().unwrap();

        match dialogs::pick_mount_point(app.window()) {
            Some(path) => path.to_string_lossy().to_shared_string(),
            None => SharedString::new(),
        }
    });

    app.global::<Rust<'_>>()
        .on_write_config(|config| config.write().into());

    app.global::<Rust<'_>>()
        .on_get_drive_info(|path| DriveInfo::from_path(&path));

    app.global::<Rust<'_>>()
        .on_delete_dir(|path| fs::remove_dir_all(path).into());

    let model_clone = model.clone();
    app.global::<Rust<'_>>().on_load_games(move |path| {
        let path = Path::new(&path);
        let new = game::scan_drive(path);
        model_clone.set_games(new);
    });

    let model_clone = model.clone();
    app.global::<Rust<'_>>().on_load_homebrew_apps(move |path| {
        let path = Path::new(&path);
        let new = homebrew_app::scan_drive(path).unwrap_or_default();
        model_clone.set_homebrew_apps(new);
    });

    let model_clone = model.clone();
    app.global::<Rust<'_>>()
        .on_load_osc_apps(move |force_refresh| {
            let (new, h, min) = osc::load_contents(force_refresh).unwrap_or_default();
            model_clone.set_osc_apps(new);
            (h, min)
        });

    let model_clone = model.clone();
    app.global::<Rust<'_>>()
        .on_filter_games(move |filter| model_clone.set_games_filter(filter));

    let model_clone = model.clone();
    app.global::<Rust<'_>>()
        .on_filter_homebrew_apps(move |filter| model_clone.set_homebrew_apps_filter(filter));

    let model_clone = model.clone();
    app.global::<Rust<'_>>()
        .on_filter_osc_apps(move |filter| model_clone.set_osc_apps_filter(filter));

    app.global::<Rust<'_>>()
        .on_get_disc_info(|game_dir| DiscInfo::try_from_game_dir(Path::new(&game_dir)).into());

    let weak = app.as_weak();
    app.global::<Rust<'_>>()
        .on_pick_games(move |existing_games| {
            let app = weak.upgrade().unwrap();
            let paths = dialogs::pick_games(app.window());
            let existing_ids = existing_games.iter().map(|g| g.id).collect::<Vec<_>>();
            let queue: Vec<QueuedConversion> =
                standard_conversion::make_queue(paths, &existing_ids);
            let model = VecModel::from(queue);
            ModelRc::from(Rc::new(model))
        });

    let weak = app.as_weak();
    app.global::<Rust<'_>>()
        .on_pick_games_r(move |existing_games| {
            let app = weak.upgrade().unwrap();
            let paths = dialogs::pick_games_r(app.window());
            let existing_ids = existing_games.iter().map(|g| g.id).collect::<Vec<_>>();
            let queue = standard_conversion::make_queue(paths, &existing_ids);
            let model = VecModel::from(queue);
            ModelRc::from(Rc::new(model))
        });

    let model_clone = model.clone();
    app.global::<Rust<'_>>()
        .on_sort(move |sort_by| model_clone.sort(sort_by));

    let model_clone = model.clone();
    app.global::<Rust<'_>>()
        .on_set_show_wii(move |show_wii| model_clone.set_show_wii(show_wii));

    let model_clone = model.clone();
    app.global::<Rust<'_>>()
        .on_set_show_gc(move |show_gc| model_clone.set_show_gc(show_gc));

    app.global::<Rust<'_>>()
        .on_add_notification(|notifications, notification| {
            notifications
                .as_any()
                .downcast_ref::<VecModel<Notification>>()
                .unwrap()
                .push(notification);
        });

    app.global::<Rust<'_>>()
        .on_close_notification(|notifications, i| {
            #[allow(clippy::cast_sign_loss)]
            notifications
                .as_any()
                .downcast_ref::<VecModel<Notification>>()
                .unwrap()
                .remove(i as usize);
        });

    app.global::<Rust<'_>>().on_add_to_queue(|queue, new| {
        let model = queue
            .as_any()
            .downcast_ref::<VecModel<QueuedConversion>>()
            .unwrap();

        let new_model = new
            .as_any()
            .downcast_ref::<VecModel<QueuedConversion>>()
            .unwrap();

        while new.row_count() > 0 {
            model.push(new_model.remove(new.row_count() - 1));
        }
    });

    let weak = app.as_weak();
    app.global::<Rust<'_>>().on_pick_archive_dest(move |game| {
        let app = weak.unwrap();

        match dialogs::save_game(app.window(), &game) {
            Some(path) => path.to_string_lossy().to_shared_string(),
            None => SharedString::new(),
        }
    });

    app.global::<Rust<'_>>()
        .on_add_to_conversion_queue(move |queue, queued| {
            queue
                .as_any()
                .downcast_ref::<VecModel<QueuedConversion>>()
                .unwrap()
                .push(queued);
        });

    let weak = app.global::<State<'_>>().as_weak();
    app.global::<Rust<'_>>()
        .on_run_conversion(move |queue, conf, drive_info| {
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
    app.global::<Rust<'_>>().on_load_osc_icons(move |_apps| {
        // TODO
        //osc::load_icons(&apps, weak.clone());
    });

    let weak = app.as_weak();
    app.global::<Rust<'_>>()
        .on_install_homebrew_apps(move |mount_point| {
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
    app.global::<Rust<'_>>().on_cache_covers(move || {
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
    app.global::<Rust<'_>>().on_checksum(move |game| {
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
        app.global::<Rust<'_>>()
            .on_set_window_color(move |is_dark| {
                let app = weak.upgrade().unwrap();
                window_color::set(app.window(), is_dark);
            });
    }

    if let Err(e) = app.run() {
        if std::env::var("SLINT_BACKEND").unwrap_or_default() == "winit-software" {
            bail!(e);
        }

        return restart_with_sw_rendering();
    }

    Ok(())
}
