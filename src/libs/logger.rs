use lazy_static::lazy_static;

use crate::service::discord_error_msg::send_debug_error_log;

#[derive(PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
    Debug,
}
static LOG_LEVEL_TO_STRING: [&str; 4] = ["Error", "Warning", "Info", "Debug"];
impl LogLevel {
    pub fn to_string(&self) -> &str {
        match self {
            LogLevel::Error => LOG_LEVEL_TO_STRING[0],
            LogLevel::Warning => LOG_LEVEL_TO_STRING[1],
            LogLevel::Info => LOG_LEVEL_TO_STRING[2],
            LogLevel::Debug => LOG_LEVEL_TO_STRING[3],
        }
    }
}
pub struct Logger {
    log_fn: fn(&str,LogLevel),
}

impl Logger {
    pub fn new(log_fn: fn(&str,LogLevel)) -> Self {
        Logger { log_fn }
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Error => (self.log_fn)(message, LogLevel::Error),
            LogLevel::Warning => (self.log_fn)(message, LogLevel::Warning),
            LogLevel::Info => (self.log_fn)(message, LogLevel::Info),
            LogLevel::Debug => if cfg!(debug_assertions) {
                (self.log_fn)(message, LogLevel::Debug)
            },
        }
    }
}
lazy_static! {
    pub static ref LOGGER: Logger = Logger::new(|message, level| {
        if level == LogLevel::Error {
            // Spawn the future so it runs in the background
            tokio::spawn(send_debug_error_log(message.to_string()));
        }
        println!("[{:?}] {}", level.to_string(), message);
    });
}