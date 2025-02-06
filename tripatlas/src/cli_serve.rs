use anyhow::Result;

use crate::start_server;

pub struct ServeParams {
    pub host: String,
    pub port: u16,
    pub on_start: Option<Box<dyn FnOnce() + Send>>,
    pub on_port_in_use: Option<Box<dyn FnOnce() + Send>>,
    pub allow_shutdown_from_frontend: bool,
}

pub async fn serve(params: ServeParams) -> Result<()> {
    let url = format!("http://{}:{}", params.host, params.port);

    if let Ok(response) = reqwest::get(&url).await {
        if response.status() == reqwest::StatusCode::OK {
            if let Some(on_port_in_use) = params.on_port_in_use {
                on_port_in_use();
            }
            return Ok(());
        }
    }

    let listener = std::net::TcpListener::bind((params.host.as_str(), params.port))?;
    println!("Starting server on {}", url);

    start_server::start_server(
        listener,
        params.on_start,
        params.allow_shutdown_from_frontend,
    )
    .await?;
    Ok(())
}
