use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;

pub struct State {
    config: Config,
    metrics: PrometheusMetrics,
}

struct PrometheusMetrics {
    registry: prometheus::Registry,
    index_html_requests_total: prometheus::Counter,
    assets_requests_total: prometheus::Counter,
    assets_not_found_total: prometheus::Counter,
    api_root_requests_total: prometheus::Counter,
    metrics_requests_total: prometheus::Counter,
    config_requests_total: prometheus::Counter,
    experimental_requests_total: prometheus::Counter,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    allow_shutdown_from_frontend: bool,
}

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

#[actix_web::get("/api/some_hash_23423/{zoom}_{tile_x}_{tile_y}.bin")]
async fn route_api_tile_color(
    state: web::Data<State>,
    path: web::Path<(u8, u32, u32)>,
) -> impl Responder {
    state.metrics.experimental_requests_total.inc();
    let (zoom, tile_x, tile_y) = path.into_inner();
    let mut rng = rand_chacha::ChaChaRng::seed_from_u64(
        (zoom as u64) * 45634541 + (tile_x as u64) * 1234567 + (tile_y as u64),
    );
    let color = format!(
        "rgba({}, {}, {}, 0.2)",
        rng.random::<u8>(),
        rng.random::<u8>(),
        rng.random::<u8>(),
    );
    HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .body(color)
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
            .service(route_api_root)
            .service(route_api_config)
            .service(route_api_shutdown)
            .service(route_api_metrics)
            .service(route_api_tile_color)
            .service(route_frontend)
    })
    .workers(1)
    .listen(listener)?;
    if let Some(on_start) = on_start {
        on_start();
    }
    server.run().await
}
