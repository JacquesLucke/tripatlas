use anyhow::Result;

mod cli;
mod cli_gtfs_merge;
mod cli_gtfs_stats;
mod cli_mobility_database;
mod cli_serve;
mod cli_serve_dev;
mod coordinates;
mod gtfs_sources;
mod projection;
mod start_server;
mod util;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<()> {
    cli::handle_command_line_arguments().await
}
