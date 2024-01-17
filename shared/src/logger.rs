use log::{debug, error, info, trace, warn, LevelFilter};
use std::io::Write;
use super::colors;

pub fn setup_logger(log_level: &str) {
    let level = match log_level {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    env_logger::Builder::new()
        .format(|buf, record| {
            let level = match record.level() {
                log::Level::Error => colors::RED,
                log::Level::Warn => colors::YELLOW,
                log::Level::Info => colors::GREEN,
                log::Level::Debug => colors::BLUE,
                log::Level::Trace => colors::CYAN,
            };
            writeln!(
                buf,
                "[{}{}{}] - {}",
                level,
                record.level(),
                colors::RESET,
                record.args()
            )
        })
        .filter(None, level)
        .target(env_logger::Target::Stdout)
        .write_style(env_logger::WriteStyle::Always)
        .init();
}