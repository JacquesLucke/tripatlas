use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use gtfs_io::GtfsFilter;
use serde::{Deserialize, Serialize};
use std::{
    net::TcpListener,
    path::Path,
    sync::{LazyLock, OnceLock},
};

use crate::gtfs_dataset::GtfsDataset;

pub struct State {
    pub config: Config,
    pub metrics: PrometheusMetrics,
    pub dataset: GtfsDataset,
}

pub struct PrometheusMetrics {
    pub registry: prometheus::Registry,
    pub index_html_requests_total: prometheus::Counter,
    pub assets_requests_total: prometheus::Counter,
    pub assets_not_found_total: prometheus::Counter,
    pub api_root_requests_total: prometheus::Counter,
    pub metrics_requests_total: prometheus::Counter,
    pub config_requests_total: prometheus::Counter,
    pub station_requests_total: prometheus::Counter,
    pub _experimental_requests_total: prometheus::Counter,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub allow_shutdown_from_frontend: bool,
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
    let station_requests_total = prometheus::Counter::with_opts(
        prometheus::Opts::new(
            "stations_requests_total",
            "Total number of stations requests",
        )
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
        &station_requests_total,
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
        station_requests_total,
        _experimental_requests_total: experimental_requests_total,
    }
}

pub async fn start_server(
    listener: TcpListener,
    on_start: Option<Box<dyn FnOnce() + Send>>,
    allow_shutdown_from_frontend: bool,
) -> std::io::Result<()> {
    static GTFS_PATH: &str = "/home/jacques/Documents/gtfs_germany";
    static GTFS_BUFFERS: LazyLock<gtfs_io::GtfsBuffers> = LazyLock::new(|| {
        gtfs_io::GtfsBuffers::from_path(Path::new(GTFS_PATH), &GtfsFilter::all()).unwrap()
    });

    // This state is shared across all worker threads.
    let state = web::Data::new(State {
        config: Config {
            allow_shutdown_from_frontend,
        },
        metrics: prepare_prometheus_metrics(),
        dataset: GtfsDataset {
            raw: gtfs_io::Gtfs::from_buffers(GTFS_BUFFERS.to_slices()).unwrap(),
            stops_tree: OnceLock::new(),
        },
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Cors::permissive())
            .service(crate::routes::api_basics::route_api_root)
            .service(crate::routes::api_basics::route_api_config)
            .service(crate::routes::api_basics::route_api_shutdown)
            .service(crate::routes::api_basics::route_api_metrics)
            .service(crate::routes::stations::route_api_stations)
            .service(crate::routes::frontend::route_frontend)
    })
    .workers(1)
    .listen(listener)?;
    if let Some(on_start) = on_start {
        on_start();
    }
    server.run().await
}
