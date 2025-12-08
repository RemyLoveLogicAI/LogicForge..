//! CLI module for Genesis Engine

pub mod commands;
pub mod output;

pub use commands::{Cli, Commands, OutputFormat};
pub use output::{
    confirm, create_progress_bar, create_spinner, print_error, print_header, print_info,
    print_success, print_warning, prompt, Formatter, JsonFormatter, TextFormatter,
};
