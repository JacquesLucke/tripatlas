use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
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

#[actix_web::get("/api/metrics")]
async fn route_api_metrics(state: web::Data<State>) -> impl Responder {
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

    let counters = vec![
        &index_html_requests_total,
        &assets_requests_total,
        &assets_not_found_total,
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
            .service(route_frontend)
    })
    .workers(1)
    .listen(listener)?;
    if let Some(on_start) = on_start {
        on_start();
    }
    server.run().await
}
