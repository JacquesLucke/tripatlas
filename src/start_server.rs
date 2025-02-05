use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;

pub struct State {
    config: Config,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    allow_shutdown_from_frontend: bool,
}

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

#[actix_web::get("/api/config")]
async fn route_api_config(state: web::Data<State>) -> impl Responder {
    HttpResponse::Ok().json(Config {
        allow_shutdown_from_frontend: state.config.allow_shutdown_from_frontend,
    })
}

#[actix_web::post("/api/shutdown")]
async fn route_api_shutdown(state: web::Data<State>) -> impl Responder {
    if !state.config.allow_shutdown_from_frontend {
        return HttpResponse::Unauthorized().body("Shutdown is not allowed.");
    }
    tokio::task::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        std::process::exit(0);
    });
    HttpResponse::Ok().body("Shutting down server.")
}

pub async fn start_server(
    listener: TcpListener,
    on_start: Option<Box<dyn FnOnce() + Send>>,
    allow_shutdown_from_frontend: bool,
) -> std::io::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(State {
                config: Config {
                    allow_shutdown_from_frontend,
                },
            }))
            .wrap(Cors::permissive())
            .service(route_api_root)
            .service(route_api_config)
            .service(route_api_shutdown)
            .service(route_frontend)
    })
    .workers(1)
    .listen(listener)?;
    if let Some(on_start) = on_start {
        on_start();
    }
    server.run().await
}
