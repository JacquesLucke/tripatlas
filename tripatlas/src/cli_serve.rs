use std::{path::PathBuf, time::Duration};

use anyhow::Result;

use crate::start_server;

pub struct ServeParams {
    pub host: String,
    pub port: u16,
    pub on_start: Option<Box<dyn FnOnce() + Send>>,
    pub on_port_in_use: Option<Box<dyn FnOnce() + Send>>,
    pub allow_shutdown_from_frontend: bool,
    pub gtfs_datasets: Vec<PathBuf>,
}

pub async fn serve(params: ServeParams) -> Result<()> {
    let url = format!("http://{}:{}", params.host, params.port);

    if website_available(&url).await {
        if let Some(on_port_in_use) = params.on_port_in_use {
            on_port_in_use();
        }
        return Ok(());
    }

    let listener = std::net::TcpListener::bind((params.host.as_str(), params.port))?;
    println!("Starting server on {}", url);

    start_server::start_server(
        listener,
        params.on_start,
        params.allow_shutdown_from_frontend,
        params.gtfs_datasets.clone(),
    )
    .await?;
    Ok(())
}

async fn website_available(url: &str) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(1000))
        .build()
        .unwrap();
    let response = client.get(url).send().await;
    match response {
        Ok(response) => response.status() == reqwest::StatusCode::OK,
        Err(_) => false,
    }
}
