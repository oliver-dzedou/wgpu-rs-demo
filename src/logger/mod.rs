use chrono::Local;
use colored::{ColoredString, Colorize};
use std::io::{stdin, IsTerminal};
use std::fmt::{Display, Formatter, Result};

#[derive(PartialEq, PartialOrd, Copy, Clone)]
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

    pub fn log_info<M>(&self, message: M) -> ()
    where
        M: AsRef<str>,
    {
        if !self.is_enabled(LogLevel::INFO) {
            return;
        }
        self.log(message, LogLevel::INFO, |s| s.bright_white())
    }

    pub fn log_warn<M>(&self, message: M) -> ()
    where
        M: AsRef<str>,
    {
        if !self.is_enabled(LogLevel::WARN) {
            return;
        }
        self.log(message, LogLevel::WARN, |s| s.bright_yellow())
    }

    pub fn log_error<M>(&self, message: M) -> ()
    where
        M: AsRef<str>,
    {
        if !self.is_enabled(LogLevel::ERROR) {
            return;
        }
        self.log(message, LogLevel::ERROR, |s| s.bright_red())
    }

    pub fn set_log_level(&mut self, log_level: LogLevel) {
        self.log_level = log_level
    }

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

    #[cfg(not(debug_assertions))]
    fn log<F>(&self, message: &str, level: LogLevel, color: F) -> ()
    where
        F: FnOnce(&str) -> ColoredString,
    {
    }

    fn is_enabled(&self, log_level: LogLevel) -> bool {
        return log_level >= self.log_level;
    }
}
