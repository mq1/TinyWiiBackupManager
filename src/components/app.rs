use crate::app::{App, ConversionState};
use crate::components;
use crate::messages::handle_messages;
use eframe::egui;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        handle_messages(self, ctx);

        match self.conversion_state {
            ConversionState::Converting { .. } => ctx.set_cursor_icon(egui::CursorIcon::Wait),
            _ => ctx.set_cursor_icon(egui::CursorIcon::Default),
        }

        components::top_panel::ui_top_panel(ctx, self);
        components::bottom_panel::ui_bottom_panel(ctx, self);

        egui::CentralPanel::default().show(ctx, |ui| {
            components::game_grid::ui_game_grid(ui, self);

            if matches!(self.conversion_state, ConversionState::Converting { .. }) {
                components::conversion_modal::ui_conversion_modal(ctx, self);
            }
        });

        // Render info windows for opened games
        self.open_info_windows.retain_mut(|&mut index| {
            self.games.get_mut(index).map_or(false, |game| {
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
        self.bottom_right_toasts.show(ctx);
    }
}
