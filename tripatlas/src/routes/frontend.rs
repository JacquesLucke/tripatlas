use actix_web::{web, HttpResponse, Responder};

use crate::start_server::State;

static FRONTEND_FILES: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/frontend/dist");

#[actix_web::get("/{filename:.*}")]
async fn route_frontend(req: actix_web::HttpRequest, state: web::Data<State>) -> impl Responder {
    let path = req.match_info().get("filename");
    let path = path.unwrap_or("not_found.html");
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = FRONTEND_FILES.get_file(path) {
        if path == "index.html" {
            state.metrics.index_html_requests_total.inc();
        } else {
            state.metrics.assets_requests_total.inc();
        }

        let mime_type = mime_guess::from_path(path).first_or_octet_stream();
        HttpResponse::Ok()
            .content_type(mime_type.to_string())
            .body(file.contents())
    } else {
        state.metrics.assets_not_found_total.inc();
        HttpResponse::NotFound().body("File not found")
    }
}
