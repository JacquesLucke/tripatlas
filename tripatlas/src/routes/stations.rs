use std::collections::HashSet;

use actix_web::{web, HttpResponse, Responder};
use rstar::AABB;

use crate::{projection::WebMercatorTile, start_server::State};

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
    state.metrics.station_requests_total.inc();
    let (zoom, tile_x, tile_y) = path.into_inner();
    let tile = WebMercatorTile::new(zoom, tile_x, tile_y);
    let tile_bounds = tile.to_bounds();

    if state.datasets.is_empty() {
        return HttpResponse::NotFound().finish();
    }

    let stops_tree = state.datasets[0].get_stops_tree();
    let all_found = stops_tree.locate_in_envelope(&AABB::from_corners(
        [tile_bounds.left, tile_bounds.top],
        [tile_bounds.right, tile_bounds.bottom],
    ));
    let close_lat = (tile_bounds.top - tile_bounds.bottom) / 256.0f32;
    let close_lon = (tile_bounds.right - tile_bounds.left) / 256.0f32;

    let mut station_groups = vec![];
    let mut handled: HashSet<u32> = HashSet::new();
    for stop in all_found {
        if handled.contains(&stop.stop_i) {
            continue;
        }

        let close_found = stops_tree.locate_in_envelope(&AABB::from_corners(
            [
                (stop.position.longitude - close_lon).max(tile_bounds.left),
                (stop.position.latitude + close_lat).min(tile_bounds.top),
            ],
            [
                (stop.position.longitude + close_lon).min(tile_bounds.right),
                (stop.position.latitude - close_lat).max(tile_bounds.bottom),
            ],
        ));
        handled.insert(stop.stop_i);
        let mut station_group = StationGroup {
            lat: stop.position.latitude,
            lon: stop.position.longitude,
            num: 1,
        };
        for close_stop in close_found {
            if handled.contains(&close_stop.stop_i) {
                continue;
            }
            handled.insert(close_stop.stop_i);
            station_group.lat = ((station_group.lat * station_group.num as f32)
                + close_stop.position.latitude)
                / (station_group.num + 1) as f32;
            station_group.lon = ((station_group.lon * station_group.num as f32)
                + close_stop.position.longitude)
                / (station_group.num + 1) as f32;
            station_group.num += 1;
        }
        station_groups.push(station_group);
    }

    HttpResponse::Ok()
        .content_type("application/json")
        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .json(station_groups)
}
