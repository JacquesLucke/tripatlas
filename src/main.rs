use anyhow::Result;

mod cli;
mod cli_serve;
mod cli_serve_dev;
mod start_server;

#[tokio::main]
async fn main() -> Result<()> {
    cli::handle_command_line_arguments().await
}
