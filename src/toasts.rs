use eframe::egui;

/// Helper function to create a styled `Toasts` instance for a specific screen corner.
pub fn create_toasts(anchor: egui_notify::Anchor) -> egui_notify::Toasts {
    egui_notify::Toasts::default()
        .with_anchor(anchor)
        .with_margin(egui::vec2(10.0, 32.0))
        .with_shadow(egui::Shadow {
            offset: [0, 0],
            blur: 0,
            spread: 1,
            color: egui::Color32::GRAY,
        })
}

/// Formats an `anyhow::Error` in multi-line format without backtrace.
pub fn format_error(e: &anyhow::Error) -> String {
    let mut msg = e.to_string();
    let mut source = e.source();
    while let Some(e) = source {
        msg.push_str(&format!("\n  Caused by: {e}"));
        source = e.source();
    }
    msg
}

/// Creates an error toast with the given message prefix and error details.
pub fn error_toast(prefix: &str, e: &anyhow::Error) -> egui_notify::Toast {
    let message = if prefix.is_empty() {
        format_error(e)
    } else {
        format!("{}: {}", prefix, format_error(e))
    };
    let mut toast = egui_notify::Toast::error(message);
    toast.duration(Some(std::time::Duration::from_secs(10)));
    toast.closable(true);
    toast
}
