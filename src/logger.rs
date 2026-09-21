use chrono::Local;
use colored::{Color, Colorize};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Info = 0,
    Warn = 1,
    Error = 2,
    Off = 3,
}

static MINIMUM_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

pub fn init_from_env() {
    let raw_level = std::env::var("COPYAST_LOG").or_else(|_| std::env::var("YUNTUNS_LOG"));
    let Ok(raw_level) = raw_level else {
        return;
    };

    let level = match raw_level.trim().to_ascii_lowercase().as_str() {
        "info" => LogLevel::Info,
        "warn" | "warning" => LogLevel::Warn,
        "error" => LogLevel::Error,
        "off" | "none" => LogLevel::Off,
        _ => return,
    };

    set_level(level);
}

pub fn set_level(level: LogLevel) {
    MINIMUM_LEVEL.store(level as u8, Ordering::Relaxed);
}

pub fn info(message: &str) {
    write_log(LogLevel::Info, "INFO", Color::Blue, message);
}

pub fn success(message: &str) {
    write_log(LogLevel::Info, " OK ", Color::Green, message);
}

pub fn warn(message: &str) {
    write_log(LogLevel::Warn, "WARN", Color::Yellow, message);
}

pub fn error(message: &str) {
    write_log(LogLevel::Error, "ERROR", Color::Red, message);
}

fn write_log(level: LogLevel, label: &str, color: Color, message: &str) {
    if (level as u8) < MINIMUM_LEVEL.load(Ordering::Relaxed) {
        return;
    }

    eprintln!(
        "{} {} {}",
        Local::now().format("%H:%M:%S").to_string().bright_black(),
        format!("[{label}]").color(color).bold(),
        message,
    );
}
