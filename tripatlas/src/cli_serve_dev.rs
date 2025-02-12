use std::path::{Path, PathBuf};

use anyhow::Result;
use tokio::process::Command;

use crate::start_server;

pub struct ServeDevParams {
    pub frontend_host: String,
    pub frontend_port: u16,
    pub api_host: String,
    pub api_port: Option<u16>,
    pub gtfs_datasets: Vec<PathBuf>,
}

pub async fn serve_dev(params: &ServeDevParams) -> Result<()> {
    if let Some(api_port) = params.api_port {
        if api_port == params.frontend_port {
            return Err(anyhow::anyhow!("Frontend and API ports must be different"));
        }
    }
    let api_listener =
        std::net::TcpListener::bind((params.api_host.as_str(), params.api_port.unwrap_or(0)))?;
    let api_port = api_listener.local_addr()?.port();

    let frontend_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("frontend");
    println!("Starting frontend dev server in {:?}", frontend_dir);

    let frontend_dev_process = Command::new("npm")
        .args([
            "run",
            "dev",
            "--",
            "--host",
            params.frontend_host.as_str(),
            "--port",
            &params.frontend_port.to_string(),
        ])
        .current_dir(frontend_dir)
        .env(
            "VITE_TRIP_ATLAS_API_URL",
            format!("http://{}:{}/api", params.api_host, api_port),
        )
        .kill_on_drop(true)
        .spawn();

    if frontend_dev_process.is_err() {
        return Err(anyhow::anyhow!(
            "Failed to start frontend dev process. This only works when you are working on the tripatlas source code."
        ));
    }

    start_server::start_server(api_listener, None, false, params.gtfs_datasets.clone()).await?;
    Ok(())
}
