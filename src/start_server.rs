use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

static FRONTEND_FILES: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

#[actix_web::get("/{filename:.*}")]
async fn route_frontend(req: actix_web::HttpRequest) -> impl Responder {
    let path = req.match_info().get("filename");
    let path = path.unwrap_or("not_found.html");
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = FRONTEND_FILES.get_file(path) {
        let mime_type = mime_guess::from_path(path).first_or_octet_stream();
        HttpResponse::Ok()
            .content_type(mime_type.to_string())
            .body(file.contents())
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

#[actix_web::get("/api")]
async fn route_api_root() -> impl Responder {
    HttpResponse::Ok().body("The api is working.")
}

pub async fn start_server(
    listener: TcpListener,
    on_start: Option<Box<dyn FnOnce() + Send>>,
) -> std::io::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .service(route_api_root)
            .service(route_frontend)
    })
    .workers(1)
    .listen(listener)?;
    if let Some(on_start) = on_start {
        on_start();
    }
    server.run().await
}
