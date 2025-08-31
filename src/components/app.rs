use crate::app::App;
use crate::components;
use crate::messages::handle_messages;
use eframe::{Storage, egui};

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        handle_messages(self, ctx);

        components::top_panel::ui_top_panel(ctx, self);
        components::bottom_panel::ui_bottom_panel(ctx, self);

        egui::CentralPanel::default().show(ctx, |ui| {
            components::game_grid::ui_game_grid(ui, self);
        });

        self.top_left_toasts.show(ctx);
        self.bottom_left_toasts.show(ctx);
        self.bottom_right_toasts.show(ctx);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        eframe::set_value(storage, "app_version", &APP_VERSION);

        if let Some(base_dir) = &self.base_dir {
            eframe::set_value(storage, "base_dir", base_dir);
        }
    }
}
