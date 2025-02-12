use std::sync::OnceLock;

use gtfs_io::Gtfs;
use rstar::{RTree, RTreeObject, AABB};

use crate::coordinates::LatLon;

pub struct GtfsDataset {
    pub raw: Gtfs<'static>,
    pub stops_tree: OnceLock<RTree<RTreeStop>>,
}

pub struct RTreeStop {
    pub stop_i: u32,
    pub position: LatLon,
}

impl RTreeObject for RTreeStop {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.position.longitude, self.position.latitude])
    }
}

impl GtfsDataset {
    pub fn get_stops_tree(&self) -> &RTree<RTreeStop> {
        self.stops_tree.get_or_init(|| {
            let stops = self.raw.stops.data.as_ref().unwrap();
            let lats = stops.stop_lat.as_ref().unwrap();
            let lons = stops.stop_lon.as_ref().unwrap();
            let mut elements = vec![];
            for (i, (lat, lon)) in lats.iter().zip(lons.iter()).enumerate() {
                let (Some(lat), Some(lon)) = (lat.0, lon.0) else {
                    continue;
                };
                let position = LatLon::new(lat, lon);
                elements.push(RTreeStop {
                    stop_i: i as u32,
                    position,
                });
            }
            RTree::bulk_load(elements)
        })
    }
}
