use eframe::egui::Context;
use std::sync::Arc;
use std::task::{Wake, Waker};

struct EguiWaker(Context);

impl Wake for EguiWaker {
    fn wake(self: Arc<Self>) {
        self.0.request_repaint();
    }
}

pub fn egui_waker(ctx: &Context) -> Waker {
    Waker::from(Arc::new(EguiWaker(ctx.clone())))
}
