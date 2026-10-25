//! On-page copy of the desktop tracing console.
//!
//! The desktop app prints these lines to the terminal. The browser has no
//! terminal, so the same events are kept here and drawn under the plots.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::Mutex;

use chrono::Local;
use eframe::egui::Color32;
use tracing::Event;
use tracing::Level;
use tracing::Metadata;
use tracing::Subscriber;
use tracing::field::Field;
use tracing::field::Visit;
use tracing::span;

/// Shortest console. A taller window gives the leftover space under the plots to it.
pub const MIN_HEIGHT: f32 = 140.0;
const MAX_LINES: usize = 500;

struct Line {
    text: String,
    color: Color32,
}

static LINES: Mutex<VecDeque<Line>> = Mutex::new(VecDeque::new());

pub struct Entry {
    pub text: String,
    pub color: Color32,
}

/// Height of the console under a plot stack of `plots` pixels.
pub fn panel_height(available: f32, plots: f32) -> f32 {
    (available - plots).max(MIN_HEIGHT)
}

pub fn snapshot() -> Vec<Entry> {
    LINES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .map(|line| Entry {
            text: line.text.clone(),
            color: line.color,
        })
        .collect()
}

pub fn init() {
    let _ = tracing::subscriber::set_global_default(ConsoleSubscriber);
}

struct ConsoleSubscriber;

impl Subscriber for ConsoleSubscriber {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        *metadata.level() <= Level::INFO
    }

    fn max_level_hint(&self) -> Option<tracing::level_filters::LevelFilter> {
        Some(tracing::level_filters::LevelFilter::INFO)
    }

    fn new_span(&self, _span: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(1)
    }

    fn record(&self, _span: &span::Id, _values: &span::Record<'_>) {}

    fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id) {}

    fn event(&self, event: &Event<'_>) {
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        let mut message = visitor.message;
        if !visitor.fields.is_empty() {
            message.push(' ');
            message.push_str(&visitor.fields);
        }
        push_line(*event.metadata().level(), event.metadata().target(), &message);
    }

    fn enter(&self, _span: &span::Id) {}

    fn exit(&self, _span: &span::Id) {}
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
    fields: String,
}

impl Visit for MessageVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            let _ = write!(self.fields, " {}={value}", field.name());
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            if self.message.is_empty() {
                self.message = format!("{value:?}");
            }
        } else {
            let _ = write!(self.fields, " {}={value:?}", field.name());
        }
    }
}

fn push_line(level: Level, target: &str, message: &str) {
    let text = format_line(level, target, message);
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&text.as_str().into());
    let mut lines = LINES.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    lines.push_back(Line {
        text,
        color: level_color(level),
    });
    while lines.len() > MAX_LINES {
        lines.pop_front();
    }
}

fn format_line(level: Level, target: &str, message: &str) -> String {
    let time = Local::now().format("%H:%M:%S%.3f");
    format!("{time} {level:<5} {target}: {message}")
}

fn level_color(level: Level) -> Color32 {
    match level {
        Level::ERROR => Color32::from_rgb(255, 96, 96),
        Level::WARN => Color32::from_rgb(255, 196, 64),
        Level::INFO => Color32::from_rgb(126, 214, 126),
        Level::DEBUG => Color32::from_rgb(120, 176, 255),
        Level::TRACE => Color32::from_rgb(168, 168, 168),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_fills_the_space_under_the_plots() {
        assert_eq!(panel_height(1_000.0, 760.0), 240.0);
        assert_eq!(panel_height(800.0, 760.0), MIN_HEIGHT);
    }

    #[test]
    fn console_line_matches_the_desktop_tracing_shape() {
        let text = format_line(Level::INFO, "km003c_lib::session", "Connected to KM003C");
        assert!(text.contains(" INFO  km003c_lib::session: Connected to KM003C"));
        assert_eq!(level_color(Level::ERROR), Color32::from_rgb(255, 96, 96));
        assert_ne!(level_color(Level::WARN), level_color(Level::INFO));
    }

    #[test]
    fn console_drops_lines_past_the_cap() {
        {
            let mut lines = LINES.lock().unwrap();
            lines.clear();
        }
        for index in 0..(MAX_LINES + 3) {
            push_line(Level::INFO, "test", &format!("line {index}"));
        }
        let lines = LINES.lock().unwrap();
        assert_eq!(lines.len(), MAX_LINES);
        assert!(lines.front().unwrap().text.contains("line 3"));
        assert!(lines.back().unwrap().text.contains(&format!("line {}", MAX_LINES + 2)));
    }
}
