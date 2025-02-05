use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::cli_serve;
use crate::cli_serve_dev;
use crate::indexed_gtfs;

const DEFAULT_FRONTEND_HOST: &str = "localhost";
const DEFAULT_FRONTEND_PORT: u16 = 7654;

#[derive(Parser, Debug)]
#[command(name = "trip-atlas")]
struct CLI {
    #[command(subcommand)]
    command: Option<CLICommand>,
}

#[derive(Subcommand, Debug)]
enum CLICommand {
    /// Start a webserver that can be accessed from a browser.
    Serve {
        #[arg(long, default_value_t = DEFAULT_FRONTEND_HOST.to_string())]
        host: String,
        #[arg(long, default_value_t = DEFAULT_FRONTEND_PORT)]
        port: u16,
    },
    /// Start a development server with live reloading for the frontend.
    Dev {
        #[arg(long, default_value_t = DEFAULT_FRONTEND_HOST.to_string())]
        host: String,
        #[arg(long, default_value_t = DEFAULT_FRONTEND_PORT)]
        port: u16,
    },
    ParseTest {},
}

pub async fn handle_command_line_arguments() -> Result<()> {
    let cli = CLI::parse();
    match cli.command {
        None => {
            let url = format!("http://{}:{}", DEFAULT_FRONTEND_HOST, DEFAULT_FRONTEND_PORT);
            let url_clone = url.clone();
            cli_serve::serve(cli_serve::ServeParams {
                host: DEFAULT_FRONTEND_HOST.to_string(),
                port: DEFAULT_FRONTEND_PORT,
                on_start: Some(Box::new(move || {
                    if !webbrowser::open(url.as_str()).is_ok() {
                        println!("Failed to open browser");
                    }
                })),
                on_port_in_use: Some(Box::new(move || {
                    println!("Server is already running on {}", url_clone);
                    if !webbrowser::open(url_clone.as_str()).is_ok() {
                        println!("Failed to open browser");
                    }
                })),
                allow_shutdown_from_frontend: true,
            })
            .await?
        }
        Some(CLICommand::Serve { host, port }) => {
            cli_serve::serve(cli_serve::ServeParams {
                host: host,
                port: port,
                on_start: None,
                on_port_in_use: None,
                allow_shutdown_from_frontend: false,
            })
            .await?
        }
        Some(CLICommand::Dev { host, port }) => {
            cli_serve_dev::serve_dev(&cli_serve_dev::ServeDevParams {
                frontend_host: host.clone(),
                frontend_port: port,
                api_host: host,
                api_port: None,
            })
            .await?
        }
        Some(CLICommand::ParseTest {}) => {
            indexed_gtfs::parse_performance_test();
        }
    }
    Ok(())
}
