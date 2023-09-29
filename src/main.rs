// SPDX-FileCopyrightText: 2023 Manuel Quarneti <manuel.quarneti@proton.me>
// SPDX-License-Identifier: GPL-2.0-only

mod wbfs_file;

use std::path::PathBuf;

use eframe::egui;
use sysinfo::{DiskExt, System, SystemExt};

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "wbfs-manager-rs",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

#[derive(Default)]
struct App {
    sys: System,
    current_disk: Option<PathBuf>,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_disks_list();
        sys.refresh_disks();

        Self {
            sys,
            current_disk: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Disks", |ui| {
                    for disk in self.sys.disks().iter().filter(|disk| disk.is_removable()) {
                        if ui.button(disk.name().to_str().unwrap()).clicked() {
                            self.current_disk = Some(disk.mount_point().to_path_buf());
                        }
                    }
                });
            });
        });

        if let Some(disk) = &self.current_disk {
            egui::TopBottomPanel::bottom("disk_info").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Disk:");
                    ui.label(disk.to_str().unwrap());
                });
            });
        }
    }
}
