use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands;
use crate::error::ScruttError;

#[derive(Debug, Parser)]
#[command(
    name = "scrutt",
    version,
    about = "local supply-chain firewall for the Node.js ecosystem"
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Shield { path: PathBuf },
}

impl Cli {
    pub fn run(self) -> Result<(), ScruttError> {
        match self.command {
            Commands::Shield { path } => commands::shield::run(&path),
        }
    }
}

pub fn run() -> Result<(), ScruttError> {
    Cli::parse().run()
}
