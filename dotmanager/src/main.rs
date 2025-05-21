mod cli;
mod config;
mod error;
mod commands;
mod loader;
mod templating;

use anyhow::Result;
use cli::Cli;
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        cli::Commands::Init => {
            commands::init::run_init()?;
        }
        cli::Commands::Apply => {
            commands::apply::run_apply()?;
        }
        cli::Commands::AddUnit { unit_name } => {
            commands::add_unit::run_add_unit(&unit_name)?;
        }
        cli::Commands::Validate => {
            commands::validate::run_validate()?;
        }
    }

    Ok(())
}
