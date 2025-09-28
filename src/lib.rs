#![doc = include_str!("../README.md")]

#[cfg(feature = "styled")]
use anstyle::{AnsiColor, Color, Style};
use log::{Level, LevelFilter, Metadata, Record};
#[cfg(feature = "env")]
use std::str::FromStr;
use std::sync::Mutex;
#[cfg(feature = "time")]
use std::time::SystemTime;
#[cfg(feature = "file")]
use std::{fs, io::Write};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

struct Logsy(Mutex<LogsyConf>);

struct LogsyConf {
    installed: bool,
    to_stderr: bool,
    #[cfg(feature = "file")]
    to_file: Option<fs::File>,
    level: Option<Level>,
}

impl log::Log for Logsy {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.0
            .lock()
            .is_ok_and(|mg| mg.level.is_some_and(|level| metadata.level() <= level))
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
        #[allow(unused_mut)]
        let Ok(mut conf) = self.0.lock() else {
            eprintln!("ERROR: unable to acquire `LogsyConf` lock, not logging this: {record:?}");
            return;
        };
        #[cfg(feature = "styled")]
        let color = match record.metadata().level() {
            Level::Trace => AnsiColor::Magenta,
            Level::Debug => AnsiColor::Blue,
            Level::Info => AnsiColor::Green,
            Level::Warn => AnsiColor::Yellow,
            Level::Error => AnsiColor::BrightRed,
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
            eprintln!("{dim}[{ts}{level} {dim}{italic}{mod_p}{italic:#}{dim}]{dim:#} {msg}");
        }
        #[cfg(feature = "file")]
        if let Some(file) = &mut conf.to_file {
            let _ = writeln!(file, "[{ts}{:5} {mod_p}] {msg}", record.level());
        }
    }
    fn flush(&self) {}
}

static LOGSY: Logsy = Logsy(Mutex::new(LogsyConf {
    installed: false,
    to_stderr: false,
    #[cfg(feature = "file")]
    to_file: None,
    level: None,
}));

/// checks whether it's already installed, does it so if not
/// # Errors
/// - if can't access global state: can't lock mutex
/// - if can't set logger
/// - if `feature(env)` and `RUST_LOG` is an invalid log level
fn ensure_installed() -> Res<()> {
    let installed = LOGSY.0.lock()?.installed;
    if !installed {
        LOGSY.0.lock()?.installed = true;
        log::set_logger(&LOGSY).map_err(|e| e.to_string())?;

        #[allow(unused_mut)] // is used if `env`
        let mut log_level = LevelFilter::Info;
        #[cfg(feature = "env")]
        if let Ok(env_log_level) = std::env::var("RUST_LOG") {
            log_level = LevelFilter::from_str(&env_log_level).unwrap_or_else(|err| {
                panic!("{err}: invalid RUST_LOG env var value: {env_log_level:?}")
            });
        }
        try_set_level(log_level)?;
    }
    Ok(())
}

/// Try to start logging to `stderr`
/// # Errors
/// - if can't `ensure_installed`
/// - if can't access global state: can't lock mutex
pub fn try_to_console() -> Res<()> {
    ensure_installed()?;
    LOGSY.0.lock()?.to_stderr = true;
    Ok(())
}

/// Start logging to `stderr`
/// # Panics
/// errors of [`try_to_console`]
pub fn to_console() {
    try_to_console().unwrap();
}

/// Try to start logging to a specified file.\
/// This function can be called again without restarting the app if you need
/// (e.g. for implementing log rotation).\
/// If parent dir doesn't exists, it will be created.
/// # Errors
/// - if can't `ensure_installed`
/// - if can't open log file
/// - if can't access global state: can't lock mutex
#[cfg(feature = "file")]
pub fn try_to_file(path: impl AsRef<std::path::Path>, append: bool) -> Res<()> {
    ensure_installed()?;

    if let Some(dirname) = path.as_ref().parent()
        && dirname.is_dir()
        && !dirname.exists()
    {
        fs::create_dir_all(dirname)?;
    }
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .open(path.as_ref())?;
    LOGSY.0.lock()?.to_file = Some(file);
    Ok(())
}

/// Start logging to a specified file.\
/// This function can be called again without restarting the app if you need
/// (e.g. for implementing log rotation).\
/// If parent dir doesn't exists, it will be created.
/// # Panics
/// errors of [`try_to_file`]
#[cfg(feature = "file")]
pub fn to_file(path: impl AsRef<std::path::Path>, append: bool) {
    try_to_file(path, append).unwrap();
}

/// Try to set log level filter
/// # Errors
/// if can't access global state: can't acquire mutex
pub fn try_set_level(filter: LevelFilter) -> Res<()> {
    log::set_max_level(filter);
    LOGSY.0.lock()?.level = filter.to_level();
    Ok(())
}

/// Set log level filter
/// # Panics
/// errors of [`try_set_level`]
pub fn set_level(filter: LevelFilter) {
    try_set_level(filter).unwrap();
}
