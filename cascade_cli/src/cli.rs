use std::path::PathBuf;

use cascade::files;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Gui,
    Dump { input: PathBuf, output: PathBuf },
}

pub fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Gui) | None => Ok(cascade_gui::run().unwrap()),
        Some(Commands::Dump { input, output }) => {
            let mut writer = files::load_writer(output)?;

            let cas = files::load_save(input).unwrap();

            serde_yaml::to_writer(&mut writer, &cas.data)?;

            Ok(())
        }
    }
}
