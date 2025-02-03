use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::middleware::DefaultHeaders;
use actix_web::{App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;

static FRONTEND_FILES: include_dir::Dir = include_dir::include_dir!("./frontend/dist");

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

pub async fn start_server(listener: TcpListener) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(DefaultHeaders::new().add(CacheControl(vec![CacheDirective::NoCache])))
            .service(route_frontend)
    })
    .workers(1)
    .listen(listener)
    .unwrap()
    .run()
    .await
}
