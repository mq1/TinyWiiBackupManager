// SPDX-FileCopyrightText: 2025 Manuel Quarneti <mq1@ik.me>
// SPDX-License-Identifier: GPL-2.0-only

#![warn(clippy::all, rust_2018_idioms)]
#![allow(non_snake_case)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use TinyWiiBackupManager::app::App;
use TinyWiiBackupManager::game::DECOMPRESSED;
use TinyWiiBackupManager::{AGENT, PRODUCT_NAME};
use anyhow::{Result, bail};
use eframe::egui;
use std::{fs, io};

const LOGO: &[u8] = include_bytes!("../assets/logo-small.png");

#[cfg(target_arch = "aarch64")]
const OPENGL_URL: &str = "https://github.com/mq1/TinyWiiBackupManager/raw/refs/tags/2.4.10/assets/mesa-llvmpipe-arm64-25.2.2/opengl32.dll.zst";

#[cfg(target_arch = "aarch64")]
const BYTE_COUNT: usize = 13404131;

#[cfg(target_arch = "aarch64")]
const DECOMPRESSED_SIZE: usize = 48907264;

#[cfg(target_arch = "x86_64")]
const OPENGL_URL: &str = "https://github.com/mq1/TinyWiiBackupManager/raw/refs/tags/2.4.10/assets/mesa-llvmpipe-x64-25.2.2/opengl32.dll.zst";

#[cfg(target_arch = "x86_64")]
const BYTE_COUNT: usize = 15945611;

#[cfg(target_arch = "x86_64")]
const DECOMPRESSED_SIZE: usize = 54465024;

#[cfg(target_arch = "x86")]
const OPENGL_URL: &str = "https://github.com/mq1/TinyWiiBackupManager/raw/refs/tags/2.4.10/assets/mesa-llvmpipe-x86-25.2.2/opengl32.dll.zst";

#[cfg(target_arch = "x86")]
const BYTE_COUNT: usize = 13320437;

#[cfg(target_arch = "x86")]
const DECOMPRESSED_SIZE: usize = 45937664;

fn download_opengl() -> Result<()> {
    let resp = AGENT.get(OPENGL_URL).call()?;

    let mut buffer = Vec::with_capacity(BYTE_COUNT);

    let (_, body) = resp.into_parts();
    let mut reader = body.into_reader();
    io::copy(&mut reader, &mut buffer)?;

    let decompressed = zstd::bulk::decompress(&buffer, DECOMPRESSED_SIZE)?;

    let app_location = std::env::current_exe()?
        .parent()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?
        .join("opengl32.dll");

    fs::write(&app_location, &decompressed)?;

    Ok(())
}

fn run(options: eframe::NativeOptions) -> Result<()> {
    if let Err(e) = eframe::run_native(
        PRODUCT_NAME,
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    ) {
        if matches!(e, eframe::Error::OpenGL(_)) && cfg!(windows) {
            let confirm = rfd::MessageDialog::new()
                .set_title("Error")
                .set_description("Failed to initialize OpenGL. Either update your graphics driver or press 'Ok' to download opengl32.dll (llvmpipe)")
                .set_buttons(rfd::MessageButtons::OkCancel)
                .set_level(rfd::MessageLevel::Error)
                .show();

            if confirm == rfd::MessageDialogResult::Ok {
                download_opengl()?;

                let _ = rfd::MessageDialog::new()
                    .set_title("Success")
                    .set_description(
                        "opengl32.dll downloaded successfully, please restart the application",
                    )
                    .set_level(rfd::MessageLevel::Info)
                    .show();

                return Ok(());
            }
        }

        bail!("{e:?}");
    }

    Ok(())
}

fn main() {
    // pre-decompress WIITDB
    std::thread::spawn(|| {
        let _ = &*DECOMPRESSED;
    });

    // Log to stderr (if you run with `RUST_LOG=debug`).
    env_logger::init();

    let icon = eframe::icon_data::from_png_bytes(LOGO).expect("Failed to load icon");
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(800.0, 600.0))
        .with_icon(icon);

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    if let Err(e) = run(options) {
        let _ = rfd::MessageDialog::new()
            .set_title("Error")
            .set_description(format!("Error: {e:?}"))
            .set_level(rfd::MessageLevel::Error)
            .show();

        std::process::exit(1);
    }
}
