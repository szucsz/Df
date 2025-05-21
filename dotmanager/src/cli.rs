use clap::{Parser, Subcommand};

/// A composable dotfile manager
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new dotfiles directory
    Init,
    /// Apply the dotfiles configuration
    Apply,
    /// Create a new dotfile unit
    AddUnit {
        unit_name: String,
    },
    /// Validate the dotfiles configuration
    Validate,
}
