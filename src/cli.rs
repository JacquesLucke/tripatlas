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
    Serve {},
    ServeDev {},
}

pub async fn handle_command_line_arguments() -> Result<()> {
    let frontend_host = "localhost";
    let frontend_port = 7654;

    let cli = CLI::parse();
    match cli.command {
        None => {
            cli_serve::serve(cli_serve::ServeParams {
                host: frontend_host.to_string(),
                port: frontend_port,
                on_start: Some(Box::new(move || {
                    if !webbrowser::open(&format!("http://{}:{}", frontend_host, frontend_port))
                        .is_ok()
                    {
                        println!("Failed to open browser");
                    }
                })),
            })
            .await?
        }
        Some(CLICommand::Serve {}) => {
            cli_serve::serve(cli_serve::ServeParams {
                host: frontend_host.to_string(),
                port: frontend_port,
                on_start: None,
            })
            .await?
        }
        Some(CLICommand::ServeDev {}) => {
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
