use lazy_static::lazy_static;
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
            LogLevel::Debug => if cfg!(not(debug_assertions)) {
                (self.log_fn)(message, LogLevel::Debug)
            },
        }
    }
}
lazy_static! {
    pub static ref LOGGER: Logger = Logger::new(|message, level| {
        println!("[{:?}] {}", level.to_string(), message);
    });
}