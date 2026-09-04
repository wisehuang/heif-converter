use std::io::Write;
use std::path::Path;

use chrono::Local;
use log::{Level, LevelFilter, Metadata, Record};

/// Logger reproducing the Python `logging` output of the original tool:
/// `%(asctime)s [%(levelname)s] %(filename)s:%(lineno)d - %(message)s`
struct StreamLogger {
    level: LevelFilter,
}

impl log::Log for StreamLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let filename = record
            .file()
            .map(|file| {
                Path::new(file)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| file.to_owned())
            })
            .unwrap_or_else(|| "unknown".to_owned());
        let mut stderr = std::io::stderr().lock();
        let _ = writeln!(
            stderr,
            "{} [{}] {}:{} - {}",
            Local::now().format("%Y-%m-%d %H:%M:%S,%3f"),
            level_name(record.level()),
            filename,
            record.line().unwrap_or(0),
            record.args()
        );
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
    }
}

fn level_name(level: Level) -> &'static str {
    match level {
        Level::Error => "ERROR",
        Level::Warn => "WARNING",
        Level::Info => "INFO",
        Level::Debug | Level::Trace => "DEBUG",
    }
}

/// Sets the logging level based on the program arguments, exactly like
/// `configure_logging` in the Python implementation.
pub fn configure(verbose: bool, extra_verbose: bool) {
    let mut level = LevelFilter::Warn;
    if verbose {
        level = LevelFilter::Info;
    }
    if extra_verbose {
        level = LevelFilter::Debug;
    }
    log::set_boxed_logger(Box::new(StreamLogger { level })).expect("logger already initialized");
    log::set_max_level(level);
}
