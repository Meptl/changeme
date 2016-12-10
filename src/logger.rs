use log::{self, LogRecord, LogLevel, LogMetadata, LogLevelFilter};
use time;

use std::io::{Stderr, stderr};
use std::io::Write;
use std::sync::{Arc, Mutex};

static HEADER: &'static str = concat!(
    "A really awesome game ",
    env!("CARGO_PKG_VERSION_MAJOR"), ".",
    env!("CARGO_PKG_VERSION_MINOR"), ".",
    env!("CARGO_PKG_VERSION_PATCH"), "\n");

// There is only one type of error in this module, a foreign error from the
// log crate.
error_chain! {
    foreign_links {
        SetLogger(log::SetLoggerError);
    }
}

/// `LoggerType` is an enum that allows us to choose between either a normal
/// `Logger<T>` or a fallback `Logger<Stderr>`.
enum LoggerType<T> where T: Write {
    Normal(Logger<T>),
    Stderr(Logger<Stderr>)
}

/// `Logger<T>` decides what type of messages are allowed to log, can be safely
/// synced between threads, and keeps track of the log timestamp.
struct Logger<T> where T: Write {
    output: Arc<Mutex<T>>,
    init_time: u64,
    level: LogLevel
}

impl<T> Logger<T> where T: Write {
    /// Attempts to create a logger that writes to an object of type `T`. In
    /// case it cannot write, we return a `LoggerType<T>` to provide a fallback
    /// to a `LoggerType<Stderr>`.
    fn new(mut output: T, level: LogLevel) -> LoggerType<T> {
        // Try to write the header to the log
        match write!(&mut output, "{}", HEADER) {
            // Success. Let's create a normal log.
            Ok(_) => {
                LoggerType::Normal(
                    Logger {
                        output: Arc::new(Mutex::new(output)),
                        init_time: time::precise_time_ns(),
                        level: level
                    }
                    )
            },
            // Failure. Let's fallback to stderr.
            Err(e) => {
                println!("Could not write to log: {}", e);
                println!("Using stderr instead");
                LoggerType::Stderr(
                    Logger {
                        output: Arc::new(Mutex::new(stderr())),
                        init_time: time::precise_time_ns(),
                        level: level
                    }
                    )
            }
        }
    }
}

unsafe impl<T> Send for Logger<T> where T: Write {}
unsafe impl<T> Sync for Logger<T> where T: Write {}

impl<T> log::Log for Logger<T> where T: Write {
    /// Checks to see if a specific message level is enabled.
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= self.level
    }

    /// Logs the record with a timestamp from the start of the `Logger`'s life.
    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            // Get difference of time since we started logging.
            let time_current = time::precise_time_ns();
            let time_delta = (time_current - self.init_time) as f64;
            let time_sec = time_delta * 0.000000001;

            let level = format!("({})", record.level());
            let msg = record.args();

            let mut out = self.output.lock().expect("Log mutex poisoned");

            match write!(out, "[{:15.5}] {:7} {}\n", time_sec, level, msg) {
                Ok(()) => (),
                Err(e) => {
                    println!("Could not write to log: {}", e);
                }
            }
        }
    }
}

/// Initializes a global logging system that can allow
pub fn init() -> Result<()> {
    Ok(try!(log::set_logger(| level | {
        level.set(LogLevelFilter::Trace);
        match Logger::new(stderr(), LogLevel::Trace) {
            LoggerType::Normal(l) => Box::new(l),
            LoggerType::Stderr(l) => Box::new(l)
        }
    })))
}
