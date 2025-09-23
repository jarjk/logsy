//!
//! `logsy` provides a simple facility for your daily logging tasks.
//! - [`set_echo`] log into stdout
//! - [`set_filename`] log into a file (can be combined with [`set_echo`])
//! - [`set_level`] set log level filter (defaults to `LevelFilter::Info`)
//!
//! Calling `set_echo` or `set_filename` for a first time gets our logging handler automatically installed.
//! # Usage
//! ```
//! use log::*;
//!
//! logsy::set_echo(true);
//! logsy::set_filename(Some("logs/main.log")).expect("Couldn't open main.log");
//!
//! info!("Application has just started");
//! warn!("Dereferencing null pointers harms");
//! error!("This application got a boo-boo and going to be terminated");
//! ```

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
    echo: bool,
    file: Option<File>,
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
            Level::Trace => Some(Color::Ansi(AnsiColor::Magenta)),
            Level::Debug => Some(Color::Ansi(AnsiColor::Blue)),
            Level::Info => Some(Color::Ansi(AnsiColor::Green)),
            Level::Warn => Some(Color::Ansi(AnsiColor::Yellow)),
            Level::Error => Some(Color::Ansi(AnsiColor::Red)),
        };

        if conf.echo {
            #[cfg(feature = "styled")]
            let [level_style, dim, italic] = {
                [
                    Style::new().fg_color(color).bold(),
                    Style::new().dimmed(),
                    Style::new().italic(),
                ]
            };
            #[cfg(not(feature = "styled"))]
            let [level_style, dim, italic] = { [String::new(), String::new(), String::new()] };
            let level = format!("{level_style}{:5}{level_style:#}", record.level());

            let ts = format!("{italic}{ts}{italic:#}");
            println!("{dim}[{ts}{dim:#}{level} {mod_p}{dim}]{dim:#} {msg}");
        }
        if let Some(file) = &mut conf.file {
            let _ = writeln!(file, "[{ts} {:5} {mod_p}] {msg}", record.level());
            let _ = file.flush();
        }
    }
    fn flush(&self) {}
}

static LOGSY: Logsy = Logsy(Mutex::new(LogsyConf {
    installed: false,
    echo: false,
    file: None,
    level: None,
}));

fn check_installed() {
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

/// Start logging into `stdout`
pub fn set_echo(echo: bool) {
    check_installed();
    LOGSY.0.lock().unwrap().echo = echo;
}

/// Start logging into a specified file.
/// This function can be called again without restarting the app if you need
/// (e.g. for implementing log rotation).
/// If parent dir doesn't exists, it's going to be created.
pub fn set_filename(filename: Option<&str>) -> Option<()> {
    check_installed();

    if let Some(filename) = filename {
        let path = Path::new(filename);
        let parent_path: &str = path.parent()?.to_str()?;
        if !parent_path.is_empty() {
            let result = std::fs::create_dir_all(parent_path);
            if let Err(err) = result {
                eprintln!("Couldn't create {parent_path}: {err}");
                return None;
            }
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(filename)
                .unwrap();
            LOGSY.0.lock().unwrap().file = Some(file);
        }
    } else {
        LOGSY.0.lock().unwrap().file = None;
    }

    Some(())
}

/// Set log level filter
pub fn set_level(filter: LevelFilter) {
    log::set_max_level(filter);
    LOGSY.0.lock().unwrap().level = filter.to_level();
}
