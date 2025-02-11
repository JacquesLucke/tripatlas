use actix_web::{web, HttpResponse, Responder};

use crate::start_server::{Config, State};

#[actix_web::get("/api")]
async fn route_api_root(state: web::Data<State>) -> impl Responder {
    state.metrics.api_root_requests_total.inc();
    HttpResponse::Ok().body("The api is working.")
}

#[actix_web::get("/api/config")]
async fn route_api_config(state: web::Data<State>) -> impl Responder {
    state.metrics.config_requests_total.inc();
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

#[actix_web::get("/api/metrics")]
async fn route_api_metrics(state: web::Data<State>) -> impl Responder {
    state.metrics.metrics_requests_total.inc();
    let encoder = prometheus::TextEncoder::new();
    let metric_families = state.metrics.registry.gather();
    let mut buffer = vec![];
    prometheus::Encoder::encode(&encoder, &metric_families, &mut buffer).unwrap();
    HttpResponse::Ok()
        .content_type(prometheus::Encoder::format_type(&encoder))
        .body(buffer)
}
