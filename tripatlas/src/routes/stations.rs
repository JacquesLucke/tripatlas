use std::collections::HashSet;

use actix_web::{web, HttpResponse, Responder};
use rstar::AABB;

use crate::{coordinates::LatLon, projection::WebMercatorTile, start_server::State};

#[derive(serde::Serialize)]
struct StationGroup {
    lat: f32,
    lon: f32,
    num: u32,
}

#[actix_web::get("/api/some_hash_23434/{zoom}_{tile_x}_{tile_y}.json")]
async fn route_api_stations(
    state: web::Data<State>,
    path: web::Path<(u8, u32, u32)>,
) -> impl Responder {
    state.metrics.experimental_requests_total.inc();
    let (zoom, tile_x, tile_y) = path.into_inner();
    let tile = WebMercatorTile::new(zoom, tile_x, tile_y);
    let tile_bounds = tile.to_bounds();

    let start = std::time::Instant::now();

    let stops_tree = state.dataset.get_stops_tree();
    let found = stops_tree.locate_in_envelope(&AABB::from_corners(
        [tile_bounds.left, tile_bounds.top],
        [tile_bounds.right, tile_bounds.bottom],
    ));
    let locations = found
        .map(|stop| StationGroup {
            lat: stop.position.latitude,
            lon: stop.position.longitude,
            num: 1,
        })
        .collect::<Vec<_>>();

    println!("Took {:?}, {:?}", start.elapsed(), locations.len());

    let Ok(result) = serde_json::to_string_pretty(&locations) else {
        return HttpResponse::NotImplemented().finish();
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .body(result)
}
