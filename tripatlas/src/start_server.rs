use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use gtfs_io::GtfsFilter;
use serde::{Deserialize, Serialize};
use std::{net::TcpListener, path::PathBuf, sync::OnceLock};

use crate::gtfs_dataset::GtfsDataset;

pub struct State {
    pub config: Config,
    pub metrics: PrometheusMetrics,
    pub datasets: Vec<GtfsDataset>,
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
    gtfs_datasets: Vec<PathBuf>,
) -> std::io::Result<()> {
    for path in &gtfs_datasets {
        println!("Loading GTFS from {:?}", path);
    }

    static CELL: OnceLock<Vec<gtfs_io::GtfsBuffers>> = OnceLock::new();
    let buffers = CELL.get_or_init(|| {
        gtfs_datasets
            .iter()
            .map(|p| gtfs_io::GtfsBuffers::from_path(&p, &GtfsFilter::all()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    });
    let datasets = buffers
        .iter()
        .map(|b| GtfsDataset {
            raw: gtfs_io::Gtfs::from_buffers(b.to_slices()).unwrap(),
            stops_tree: OnceLock::new(),
        })
        .collect::<Vec<_>>();

    // This state is shared across all worker threads.
    let state = web::Data::new(State {
        config: Config {
            allow_shutdown_from_frontend,
        },
        metrics: prepare_prometheus_metrics(),
        datasets,
    });

    let server = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Cors::permissive())
            .wrap(actix_web::middleware::Compress::default())
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
