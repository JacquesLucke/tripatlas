use anyhow::Result;

use crate::start_server;

pub async fn serve() -> Result<()> {
    let port = 7654;
    let host = "0.0.0.0";
    let listener = std::net::TcpListener::bind((host, port))?;
    let port = listener.local_addr()?.port();
    println!("Server running on http://{host}:{port}");

    start_server::start_server(listener).await?;
    Ok(())
}
