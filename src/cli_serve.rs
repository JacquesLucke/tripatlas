use anyhow::Result;

use crate::start_server;

pub struct ServeParams {
    pub host: String,
    pub port: u16,
}

pub async fn serve(params: &ServeParams) -> Result<()> {
    let listener = std::net::TcpListener::bind((params.host.as_str(), params.port))?;
    let port = listener.local_addr()?.port();
    println!("Server running on http://{}:{}", params.host, port);

    start_server::start_server(listener).await?;
    Ok(())
}
