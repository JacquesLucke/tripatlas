use anyhow::Result;
use indexed_gtfs::{Gtfs, GtfsFilter};
use rstar::{RTree, RTreeObject, AABB};
use std::path::Path;

use crate::{
    coordinates::{LatLon, XYZ},
    gtfs_sources,
};

#[derive(Debug)]
struct OriginalStop {
    source_i: u32,
    stop_i: u32,
    /// Use 3D coordinates here to handle wraparounds of longitudes.
    position: XYZ,
}

impl RTreeObject for OriginalStop {
    type Envelope = AABB<[f32; 3]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.position.x, self.position.y, self.position.z])
    }
}

pub async fn gtfs_merge(input_path: &Path, output_path: &Path) -> Result<()> {
    let gtfs_sources = gtfs_sources::get_gtfs_sources(input_path, true);
    let gtfs_sources = gtfs_sources::sort_gtfs_sources_by_size(gtfs_sources);
    println!("Found {} GTFS sources to merge.", gtfs_sources.len());

    let mut all_original_stops = Vec::new();

    for (source_i, gtfs_source) in gtfs_sources.iter().enumerate() {
        println!("{: >3} Loading GTFS from {:?}", source_i + 1, gtfs_source);
        let Ok(buffers) = indexed_gtfs::GtfsBuffers::from_path(
            &gtfs_source,
            &GtfsFilter {
                stops: true,
                ..GtfsFilter::none()
            },
        ) else {
            continue;
        };
        let Ok(gtfs) = indexed_gtfs::Gtfs::from_buffers(buffers.to_slices()) else {
            continue;
        };
        let Some(stops) = gtfs.stops.data else {
            continue;
        };
        let (Some(longitudes), Some(latitudes)) = (stops.stop_lon, stops.stop_lat) else {
            continue;
        };
        for (stop_i, (longitude, latitude)) in longitudes.iter().zip(latitudes.iter()).enumerate() {
            let (Some(longitude), Some(latitude)) = (longitude.0, latitude.0) else {
                continue;
            };
            let position = LatLon::new(latitude, longitude).to_xyz_km();
            all_original_stops.push(OriginalStop {
                source_i: source_i as u32,
                stop_i: stop_i as u32,
                position,
            });
        }
    }

    let start = std::time::Instant::now();
    println!("Size: {}", all_original_stops.len());
    let tree = RTree::bulk_load(all_original_stops);
    println!("Time to build tree: {:?}", start.elapsed());

    let a = LatLon::new(52.6374196, 13.2054151).to_xyz_km();
    let size_km = 1.0f32;
    let b = AABB::from_corners(
        [a.x - size_km, a.y - size_km, a.z - size_km],
        [a.x + size_km, a.y + size_km, a.z + size_km],
    );

    for stop in tree.locate_in_envelope(&b) {
        let source = &gtfs_sources[stop.source_i as usize];
        let Ok(buffers) = indexed_gtfs::GtfsBuffers::from_path(
            &source,
            &GtfsFilter {
                stops: true,
                attributions: true,
                ..GtfsFilter::none()
            },
        ) else {
            continue;
        };
        let gtfs = Gtfs::from_buffers(buffers.to_slices()).unwrap();
        let stop = gtfs
            .stops
            .data
            .as_ref()
            .unwrap()
            .stop_name
            .as_ref()
            .unwrap()
            .get(stop.stop_i as usize)
            .unwrap();
        println!("{:?}", stop);
    }

    // println!("{:#?}", tree);

    Ok(())
}
