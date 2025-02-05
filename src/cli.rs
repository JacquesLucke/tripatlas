use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::cli_serve;
use crate::cli_serve_dev;

#[derive(Parser, Debug)]
#[command(name = "trip-atlas")]
struct CLI {
    #[command(subcommand)]
    command: Option<CLICommand>,
}

#[derive(Subcommand, Debug)]
enum CLICommand {
    /// Start a webserver that can be accessed from a browser.
    Serve {},
    /// Start a development server with live reloading for the frontend.
    Dev {},
}

pub async fn handle_command_line_arguments() -> Result<()> {
    let frontend_host = "localhost";
    let frontend_port = 7654;

    let cli = CLI::parse();
    match cli.command {
        None => {
            let url = format!("http://{}:{}", frontend_host, frontend_port);
            let url_clone = url.clone();
            cli_serve::serve(cli_serve::ServeParams {
                host: frontend_host.to_string(),
                port: frontend_port,
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
        Some(CLICommand::Serve {}) => {
            cli_serve::serve(cli_serve::ServeParams {
                host: frontend_host.to_string(),
                port: frontend_port,
                on_start: None,
                on_port_in_use: None,
                allow_shutdown_from_frontend: false,
            })
            .await?
        }
        Some(CLICommand::Dev {}) => {
            cli_serve_dev::serve_dev(&cli_serve_dev::ServeDevParams {
                frontend_host: frontend_host.to_string(),
                frontend_port: frontend_port,
                api_host: "0.0.0.0".to_string(),
                api_port: 7777,
            })
            .await?
        }
    }
    Ok(())
}
