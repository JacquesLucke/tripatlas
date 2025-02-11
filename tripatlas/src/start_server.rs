use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;

use crate::{coordinates::LatLon, projection::WebMercatorTile};

pub struct State {
    pub config: Config,
    pub metrics: PrometheusMetrics,
}

pub struct PrometheusMetrics {
    pub registry: prometheus::Registry,
    pub index_html_requests_total: prometheus::Counter,
    pub assets_requests_total: prometheus::Counter,
    pub assets_not_found_total: prometheus::Counter,
    pub api_root_requests_total: prometheus::Counter,
    pub metrics_requests_total: prometheus::Counter,
    pub config_requests_total: prometheus::Counter,
    pub experimental_requests_total: prometheus::Counter,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub allow_shutdown_from_frontend: bool,
}

#[actix_web::get("/api/some_hash_23424/{zoom}_{tile_x}_{tile_y}.bin")]
async fn route_api_tile_color(
    state: web::Data<State>,
    path: web::Path<(u8, u32, u32)>,
) -> impl Responder {
    state.metrics.experimental_requests_total.inc();
    let (zoom, tile_x, tile_y) = path.into_inner();
    let tile = WebMercatorTile::new(zoom, tile_x, tile_y);
    let tile_bounds = tile.to_bounds();

    let my_pos = LatLon::new(52.637641, 13.205084);
    let color = if tile_bounds.contains(my_pos) {
        "rgba(255,0,0,0.2)"
    } else {
        "rgba(0,0,0,0.2)"
    };

    HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .body(color)
}

fn prepare_prometheus_metrics() -> PrometheusMetrics {
    let namespace = "tripatlas";
    let index_html_requests_total = prometheus::Counter::with_opts(
        prometheus::Opts::new(
            "index_html_requests_total",
            "Total number of index.html requests",
        )
        .namespace(namespace),
    )
    .unwrap();
    let assets_requests_total = prometheus::Counter::with_opts(
        prometheus::Opts::new("assets_requests_total", "Total number of assets requests")
            .namespace(namespace),
    )
    .unwrap();
    let assets_not_found_total = prometheus::Counter::with_opts(
        prometheus::Opts::new("assets_not_found_total", "Total number of assets not found")
            .namespace(namespace),
    )
    .unwrap();
    let api_root_requests_total = prometheus::Counter::with_opts(
        prometheus::Opts::new(
            "api_root_requests_total",
            "Total number of api root requests",
        )
        .namespace(namespace),
    )
    .unwrap();
    let metrics_requests_total = prometheus::Counter::with_opts(
        prometheus::Opts::new("metrics_requests_total", "Total number of metrics requests")
            .namespace(namespace),
    )
    .unwrap();
    let config_requests_total = prometheus::Counter::with_opts(
        prometheus::Opts::new("config_requests_total", "Total number of config requests")
            .namespace(namespace),
    )
    .unwrap();

    let experimental_requests_total = prometheus::Counter::with_opts(
        prometheus::Opts::new(
            "experimental_requests_total",
            "Total number of experimental requests",
        )
        .namespace(namespace),
    )
    .unwrap();

    let counters = vec![
        &index_html_requests_total,
        &assets_requests_total,
        &assets_not_found_total,
        &api_root_requests_total,
        &metrics_requests_total,
        &config_requests_total,
        &experimental_requests_total,
    ];

    let registry = prometheus::Registry::new();
    for counter in counters {
        registry.register(Box::new(counter.clone())).unwrap();
    }

    PrometheusMetrics {
        registry,
        index_html_requests_total,
        assets_requests_total,
        assets_not_found_total,
        api_root_requests_total,
        metrics_requests_total,
        config_requests_total,
        experimental_requests_total,
    }
}

pub async fn start_server(
    listener: TcpListener,
    on_start: Option<Box<dyn FnOnce() + Send>>,
    allow_shutdown_from_frontend: bool,
) -> std::io::Result<()> {
    // This state is shared across all worker threads.
    let state = web::Data::new(State {
        config: Config {
            allow_shutdown_from_frontend,
        },
        metrics: prepare_prometheus_metrics(),
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Cors::permissive())
            .service(crate::routes::api_basics::route_api_root)
            .service(crate::routes::api_basics::route_api_config)
            .service(crate::routes::api_basics::route_api_shutdown)
            .service(crate::routes::api_basics::route_api_metrics)
            .service(route_api_tile_color)
            .service(crate::routes::frontend::route_frontend)
    })
    .workers(1)
    .listen(listener)?;
    if let Some(on_start) = on_start {
        on_start();
    }
    server.run().await
}
