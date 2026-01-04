pub mod messages;
pub mod config;

pub use messages::*;
pub use config::*;

pub const COMMAND_CHANNEL: &str = "manager:commands";
pub const RESPONSE_CHANNEL: &str = "manager:responses";