use anyhow::Result;

mod cli;
mod cli_serve;
mod start_server;

#[tokio::main]
async fn main() -> Result<()> {
    cli::handle_command_line_arguments().await
}
