use anyhow::Result;

mod cli;
mod cli_serve;
mod cli_serve_dev;
mod csv_parse;
mod indexed_gtfs;
mod start_server;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    cli::handle_command_line_arguments().await
}
