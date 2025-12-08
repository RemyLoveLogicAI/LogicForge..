//! Logging setup and utilities

use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Log level configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

impl From<bool> for LogLevel {
    fn from(verbose: bool) -> Self {
        if verbose {
            LogLevel::Debug
        } else {
            LogLevel::Info
        }
    }
}

/// Initialize the logging system
pub fn init_logger(verbose: bool) {
    let level = if verbose { "debug" } else { "info" };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("genesis_engine={},genesis={}", level, level)));

    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::CLOSE)
        .compact();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

/// Initialize logger with JSON output (for production)
pub fn init_json_logger(verbose: bool) {
    let level = if verbose { "debug" } else { "info" };

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("genesis_engine={},genesis={}", level, level)));

    let fmt_layer = fmt::layer()
        .json()
        .with_span_events(FmtSpan::CLOSE);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

/// Create a span for operation tracking
#[macro_export]
macro_rules! operation_span {
    ($name:expr) => {
        tracing::info_span!("operation", name = $name)
    };
    ($name:expr, $($field:tt)*) => {
        tracing::info_span!("operation", name = $name, $($field)*)
    };
}

/// Log timing for an operation
pub struct OperationTimer {
    name: String,
    start: std::time::Instant,
}

impl OperationTimer {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        tracing::debug!("Starting operation: {}", name);
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }

    pub fn finish(self) {
        let duration = self.start.elapsed();
        tracing::info!(
            "Operation '{}' completed in {:?}",
            self.name,
            duration
        );
    }

    pub fn finish_with_result<T, E: std::fmt::Display>(self, result: &Result<T, E>) {
        let duration = self.start.elapsed();
        match result {
            Ok(_) => {
                tracing::info!(
                    "Operation '{}' succeeded in {:?}",
                    self.name,
                    duration
                );
            }
            Err(e) => {
                tracing::error!(
                    "Operation '{}' failed in {:?}: {}",
                    self.name,
                    duration,
                    e
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(Level::from(LogLevel::Debug), Level::DEBUG);
        assert_eq!(Level::from(LogLevel::Info), Level::INFO);
    }

    #[test]
    fn test_verbose_flag() {
        assert_eq!(LogLevel::from(true), LogLevel::Debug);
        assert_eq!(LogLevel::from(false), LogLevel::Info);
    }
}
