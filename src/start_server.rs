use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::middleware::DefaultHeaders;
use actix_web::{App, HttpServer};
use std::net::TcpListener;

#[actix_web::get("/")]
async fn route_index() -> &'static str {
    "Hey there!"
}

pub async fn start_server(listener: TcpListener) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(DefaultHeaders::new().add(CacheControl(vec![CacheDirective::NoCache])))
            .service(route_index)
    })
    .workers(1)
    .listen(listener)
    .unwrap()
    .run()
    .await
}
