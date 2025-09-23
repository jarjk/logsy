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

use chrono::Local;
use log::{Level, LevelFilter, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

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
        let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
        let mut conf = self.0.lock().unwrap();
        let color = match record.metadata().level() {
            Level::Error | Level::Warn => Some(Color::Red),
            Level::Info => Some(Color::Green),
            _ => None,
        };

        if conf.echo {
            let mut stdout = StandardStream::stdout(ColorChoice::Always);
            print!("[{ts}][");
            let _ = stdout.set_color(ColorSpec::new().set_fg(color));
            print!("{}", record.level());
            let _ = stdout.reset();
            println!("] {}", record.args());
        }
        if let Some(file) = &mut conf.file {
            let _ = writeln!(file, "[{}][{}] {}", ts, record.level(), record.args());
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
        set_level(LevelFilter::Info);
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
