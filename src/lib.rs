#![doc = include_str!("../README.md")]

#[cfg(feature = "styled")]
use anstyle::{AnsiColor, Color, Style};
use log::{Level, LevelFilter, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
#[cfg(feature = "env")]
use std::str::FromStr;
use std::sync::Mutex;
#[cfg(feature = "time")]
use std::time::SystemTime;

struct Logsy(Mutex<LogsyConf>);

struct LogsyConf {
    installed: bool,
    to_stderr: bool,
    to_file: Option<File>,
    level: Option<Level>,
}

impl log::Log for Logsy {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if let Some(level) = self.0.lock().unwrap().level {
            metadata.level() <= level
        } else {
            false
        }
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        #[cfg(feature = "time")]
        let ts = format!("{} ", humantime::format_rfc3339_micros(SystemTime::now())); // notice the extra space
        #[cfg(not(feature = "time"))]
        let ts = String::new();
        let mod_p = record.module_path().unwrap_or_default();
        let msg = record.args();
        let mut conf = self.0.lock().unwrap();
        #[cfg(feature = "styled")]
        let color = match record.metadata().level() {
            Level::Trace => AnsiColor::Magenta,
            Level::Debug => AnsiColor::Blue,
            Level::Info => AnsiColor::Green,
            Level::Warn => AnsiColor::Yellow,
            Level::Error => AnsiColor::Red,
        };

        if conf.to_stderr {
            #[cfg(feature = "styled")]
            let [level_style, dim, italic] = {
                [
                    Style::new().fg_color(Some(Color::Ansi(color))).bold(),
                    Style::new().dimmed(),
                    Style::new().italic(),
                ]
            };
            #[cfg(not(feature = "styled"))]
            let [level_style, dim, italic] = { [String::new(), String::new(), String::new()] };
            let level = format!("{level_style}{:5}{level_style:#}", record.level());

            let ts = format!("{italic}{ts}{italic:#}");
            eprintln!("{dim}[{ts}{dim:#}{level} {mod_p}{dim}]{dim:#} {msg}");
        }
        if let Some(file) = &mut conf.to_file {
            let _ = writeln!(file, "[{ts} {:5} {mod_p}] {msg}", record.level());
            let _ = file.flush();
        }
    }
    fn flush(&self) {}
}

static LOGSY: Logsy = Logsy(Mutex::new(LogsyConf {
    installed: false,
    to_stderr: false,
    to_file: None,
    level: None,
}));

/// checks whether it's already installed, does it so if not
/// # Panics
/// - if can't access global state: can't lock mutex
/// - if can't set logger
/// - if `feature(env)` and `RUST_LOG` is an invalid log level
fn ensure_installed() {
    let installed = LOGSY.0.lock().unwrap().installed;
    if !installed {
        LOGSY.0.lock().unwrap().installed = true;
        log::set_logger(&LOGSY).unwrap();

        #[allow(unused_mut)] // is used if `env`
        let mut log_level = LevelFilter::Info;
        #[cfg(feature = "env")]
        if let Ok(env_log_level) = std::env::var("RUST_LOG") {
            log_level = LevelFilter::from_str(&env_log_level).unwrap_or_else(|err| {
                panic!("{err}: invalid RUST_LOG env var value: {env_log_level:?}")
            });
        }
        set_level(log_level);
    }
}

/// Start logging to `stderr`
/// # Panics
/// - if can't `ensure_installed`
/// - if can't access global state: can't lock mutex
pub fn to_console() {
    ensure_installed();
    LOGSY.0.lock().unwrap().to_stderr = true;
}

/// Start logging into a specified file.
/// This function can be called again without restarting the app if you need
/// (e.g. for implementing log rotation).
/// If parent dir doesn't exists, it will be created.
/// # Panics
/// - if can't `ensure_installed`
/// - if can't open log file
/// - if can't access global state: can't lock mutex
pub fn to_file(path: impl AsRef<Path>, append: bool) {
    ensure_installed();

    if let Some(dirname) = path.as_ref().parent()
        && !dirname.exists()
    {
        std::fs::create_dir(dirname)
            .unwrap_or_else(|err| panic!("couldn't create {dirname:?}: {err}"));
    }
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .open(path)
        .unwrap();
    LOGSY.0.lock().unwrap().to_file = Some(file);
}

/// Set log level filter
/// # Panics
/// if can't access global state: can't lock mutex
pub fn set_level(filter: LevelFilter) {
    log::set_max_level(filter);
    LOGSY.0.lock().unwrap().level = filter.to_level();
}
