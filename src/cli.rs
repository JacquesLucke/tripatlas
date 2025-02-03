use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::cli_serve;

#[derive(Parser, Debug)]
#[command(name = "trip-atlas")]
struct CLI {
    #[command(subcommand)]
    command: CLICommand,
}

#[derive(Subcommand, Debug)]
enum CLICommand {
    Serve {},
}

pub async fn handle_command_line_arguments() -> Result<()> {
    let cli = CLI::parse();
    match cli.command {
        CLICommand::Serve {} => cli_serve::serve().await?,
    }
    Ok(())
}
