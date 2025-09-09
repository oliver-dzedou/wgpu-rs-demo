use chrono::Local;
use colored::{ColoredString, Colorize};
use std::io::{stdin, IsTerminal};
use std::fmt::{Display, Formatter, Result};

#[derive(PartialEq, PartialOrd, Copy, Clone)]
/// Contains the available log levels for configuration of [Logger]
///
/// See [crate::Logger]
pub enum LogLevel {
    INFO,
    WARN,
    ERROR,
    NONE,
}
impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::INFO => write!(f, "INFO"),
            Self::WARN => write!(f, "WARN"),
            Self::ERROR => write!(f, "ERROR"),
            Self::NONE => write!(f, "NONE"),
        }
    }
}
/// Provides logging utility
pub struct Logger {
    is_terminal: bool,
    log_level: LogLevel,
}

impl Logger {
    pub(crate) fn new() -> Self {
        Self {
            is_terminal: stdin().is_terminal(),
            log_level: LogLevel::INFO,
        }
    }

    /// Logs an info message to TTY
    ///
    /// Example format: ``2025-04-11 08:33:55 EXUPERY::INFO <message>``
    ///
    /// Has no effect if [Self::log_level] is [LogLevel::WARN] or higher
    ///
    /// Has no effect when compiled in release mode
    ///
    /// Has no effect in a GUI environment
    ///
    /// Use the [log_info!] convenience macro over this method to log file + line number
    pub fn log_info<M>(&self, message: M) -> ()
    where
        M: AsRef<str>,
    {
        if !self.is_enabled(LogLevel::INFO) {
            return;
        }
        self.log(message, LogLevel::INFO, |s| s.bright_white())
    }

    /// Logs a warning message to TTY
    ///
    /// Example format: ``2025-04-11 08:33:55 EXUPERY::WARN <message>``
    ///
    /// Has no effect if [Self::log_level] is [LogLevel::ERROR] or higher
    ///
    /// Has no effect when compiled in release mode
    ///
    /// Has no effect in a GUI environment
    ///
    /// Use the [log_warn!] convenience macro over this method to log file + line number
    pub fn log_warn<M>(&self, message: M) -> ()
    where
        M: AsRef<str>,
    {
        if !self.is_enabled(LogLevel::WARN) {
            return;
        }
        self.log(message, LogLevel::WARN, |s| s.bright_yellow())
    }

    /// Logs an error message to TTY
    ///
    /// Example format: ``2025-04-11 08:33:55 EXUPERY::ERROR <message>``
    ///
    /// Has no effect if [Self::log_level] is [None] or higher
    ///
    /// Has no effect when compiled in release mode
    ///
    /// Has no effect in a GUI environment
    ///
    /// Use the [log_error!] convenience macro over this method to log file + line number
    pub fn log_error<M>(&self, message: M) -> ()
    where
        M: AsRef<str>,
    {
        if !self.is_enabled(LogLevel::ERROR) {
            return;
        }
        self.log(message, LogLevel::ERROR, |s| s.bright_red())
    }

    /// Sets [LogLevel]
    pub fn set_log_level(&mut self, log_level: LogLevel) {
        self.log_level = log_level
    }

    /// Writes the specified message to TTY
    ///
    /// Has no effect in a GUI environment
    ///
    /// Has no effect in release mode
    #[cfg(debug_assertions)]
    fn log<F, M>(&self, message: M, level: LogLevel, color: F) -> ()
    where
        F: FnOnce(&str) -> ColoredString,
        M: AsRef<str>,
    {
        use std::fmt::format;

        if !self.is_terminal {
            return;
        }
        let formatted = format!(
            "{} :: {} ::  {}",
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            level,
            message.as_ref()
        );
        let colored = color(formatted.as_str());
        println!("{}", colored);
    }

    /// Writes the specified message to TTY
    ///
    /// Has no effect in a GUI environment
    ///
    /// Has no effect in release mode
    #[cfg(not(debug_assertions))]
    fn log<F>(&self, message: &str, level: LogLevel, color: F) -> ()
    where
        F: FnOnce(&str) -> ColoredString,
    {
    }

    /// Checks if the given [LogLevel] is enabled
    fn is_enabled(&self, log_level: LogLevel) -> bool {
        return log_level >= self.log_level;
    }
}
