use std::fmt;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::{Context, Layer};
use tracing_subscriber::registry::LookupSpan;

// A layer that uses `termcolor` to format and print events.
pub struct TermcolorFmtLayer;

impl<S> Layer<S> for TermcolorFmtLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        // Get a handle to standard output.
        let stdout = StandardStream::stdout(ColorChoice::Auto);

        // Lock the stream for thread-safe writing.
        {
            let mut handle = stdout.lock();

            let metadata = event.metadata();
            let level = *metadata.level();

            // Set the color based on the event's level.
            let color = match level {
                Level::INFO => Color::Green,
                Level::DEBUG => Color::Blue,
                Level::WARN => Color::Yellow,
                Level::ERROR => Color::Red,
                Level::TRACE => Color::Magenta,
            };

            // Set the color and write the level string.
            handle
                .set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))
                .unwrap();
            write!(&mut handle, "{:>5}", level.as_str()).unwrap();
            handle.reset().unwrap(); // Reset to default colors.

            // Write the target and the message.
            let mut message = String::new();
            let mut visitor = MessageVisitor {
                message: &mut message,
            };
            event.record(&mut visitor);

            // Add an empty space if there's no message to avoid `target:` ending abruptly
            if message.is_empty() {
                writeln!(&mut handle, " {}", metadata.target()).unwrap();
            } else {
                writeln!(&mut handle, " {}: {}", metadata.target(), message).unwrap();
            }
        }
    }
}

// A simple visitor that extracts the "message" field's value.
struct MessageVisitor<'a> {
    message: &'a mut String,
}

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        // For this minimal logger, we only extract the "message" field.
        // Other fields (like `user_id` in `warn!(user_id=123, ...)`) will be ignored.
        if field.name() == "message" {
            // Append the formatted value to the message string.
            self.message.push_str(&format!("{:?}", value));
        }
    }
}

// Helper function to parse a string into a tracing::Level
pub fn parse_level_from_str(s: &str) -> Option<Level> {
    match s.to_ascii_lowercase().as_str() {
        "error" => Some(Level::ERROR),
        "warn" => Some(Level::WARN),
        "info" => Some(Level::INFO),
        "debug" => Some(Level::DEBUG),
        "trace" => Some(Level::TRACE),
        _ => None,
    }
}
