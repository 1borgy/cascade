#![windows_subsystem = "windows"]
#![feature(error_generic_member_access)]

use clap::{Parser, Subcommand};
use error::CascadeCliError;

mod error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// TODO: implement actual CLI commands
#[derive(Subcommand)]
enum Commands {
    Gui,
}

pub fn main() -> Result<(), CascadeCliError> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Gui) | None => Ok(cascade_gui::run()?),
    }
}
