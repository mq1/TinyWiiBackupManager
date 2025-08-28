use crate::app::App;
use crate::components;
use crate::messages::handle_messages;
use eframe::{Storage, egui};

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        handle_messages(self, ctx);

        components::top_panel::ui_top_panel(ctx, self);
        components::bottom_panel::ui_bottom_panel(ctx, self);

        egui::CentralPanel::default().show(ctx, |ui| {
            components::game_grid::ui_game_grid(ui, self);
        });

        // Render info windows for opened games and remove closed ones
        self.open_info_windows.retain(|&index| {
            self.games.get_mut(index).is_some_and(|game| {
                let mut is_open = true;
                components::game_info::ui_game_info_window(
                    ctx,
                    game,
                    &mut is_open,
                    self.inbox.sender(),
                );
                is_open
            })
        });

        self.top_left_toasts.show(ctx);
        self.bottom_left_toasts.show(ctx);
        self.bottom_right_toasts.show(ctx);
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        if let Some(base_dir) = &self.base_dir {
            eframe::set_value(storage, "base_dir", base_dir);
        }
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }
}
