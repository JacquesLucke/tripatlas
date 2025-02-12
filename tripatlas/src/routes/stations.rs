use actix_web::{web, HttpResponse, Responder};

use crate::{coordinates::LatLon, projection::WebMercatorTile, start_server::State};

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

    let mut locations = vec![];
    let Some(stops) = &state.dataset.raw.stops.data else {
        return HttpResponse::NotImplemented().finish();
    };
    let (Some(stop_lats), Some(stop_lons)) = (&stops.stop_lat, &stops.stop_lon) else {
        return HttpResponse::NotImplemented().finish();
    };

    for (lat, lon) in stop_lats.iter().zip(stop_lons.iter()) {
        let (Some(lat), Some(lon)) = (lat.0, lon.0) else {
            continue;
        };
        let position = LatLon::new(lat, lon);
        if tile_bounds.contains(position) {
            locations.push(position);
        }
    }

    println!("Took {:?}", start.elapsed());

    let Ok(result) = serde_json::to_string_pretty(&locations) else {
        return HttpResponse::NotImplemented().finish();
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .insert_header(("Cache-Control", "public, max-age=31536000, immutable"))
        .body(result)
}
