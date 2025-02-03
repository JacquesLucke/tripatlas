use anyhow::Result;
use tokio::process::Command;

use crate::start_server;

pub struct ServeDevParams {
    pub frontend_host: String,
    pub frontend_port: u16,
    pub api_host: String,
    pub api_port: u16,
}

pub async fn serve_dev(params: &ServeDevParams) -> Result<()> {
    if params.frontend_port == params.api_port {
        return Err(anyhow::anyhow!("Frontend and API ports must be different"));
    }
    Command::new("npm")
        .args([
            "run",
            "dev",
            "--",
            "--host",
            params.frontend_host.as_str(),
            "--port",
            &params.frontend_port.to_string(),
        ])
        .current_dir("./frontend")
        .env(
            "VITE_TRIP_ATLAS_API_URL",
            format!("http://{}:{}/api", params.api_host, params.api_port),
        )
        .spawn()?;

    let listener = std::net::TcpListener::bind((params.api_host.as_str(), params.api_port))?;
    start_server::start_server(listener).await?;
    Ok(())
}
