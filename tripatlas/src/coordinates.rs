/// The value only has to be in the right ballpark so that distance computations
/// on the 3D coordinates are somewhat accurate.
const APPROXIMATE_EARTH_RADIUS_IN_KM: f32 = 6378.0f32;

#[derive(Debug, Clone, Copy)]
pub struct LatLon {
    pub latitude: f32,
    pub longitude: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct XYZ {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl LatLon {
    pub fn new(latitude: f32, longitude: f32) -> Self {
        Self {
            latitude,
            longitude,
        }
    }

    pub fn to_xyz_km(&self) -> XYZ {
        let lat = to_radians(self.latitude);
        let lon = to_radians(self.longitude);

        let x = lon.cos() * lat.cos();
        let y = lon.sin() * lat.cos();
        let z = lat.sin();

        XYZ::new(
            x * APPROXIMATE_EARTH_RADIUS_IN_KM,
            y * APPROXIMATE_EARTH_RADIUS_IN_KM,
            z * APPROXIMATE_EARTH_RADIUS_IN_KM,
        )
    }
}

impl XYZ {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    #[cfg(test)]
    pub fn to_lat_lon(&self) -> LatLon {
        let x = self.x / APPROXIMATE_EARTH_RADIUS_IN_KM;
        let y = self.y / APPROXIMATE_EARTH_RADIUS_IN_KM;
        let z = self.z / APPROXIMATE_EARTH_RADIUS_IN_KM;

        let lon = to_degrees(y.atan2(x));
        let lat = to_degrees(z.asin());

        LatLon::new(lat, lon)
    }
    #[cfg(test)]
    pub fn dist_to(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;

        let d = dx * dx + dy * dy + dz * dz;
        d.sqrt()
    }
}

fn to_radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}
#[cfg(test)]
fn to_degrees(radians: f32) -> f32 {
    radians * 180.0 / std::f32::consts::PI
}

#[cfg(test)]
mod tests {
    use rand::{Rng, SeedableRng};

    use super::*;

    #[test]
    fn test_approximate_distance() {
        let a = LatLon::new(52.6374196, 13.2054151);
        let b = LatLon::new(52.6370186, 13.2055547);

        let distance_m = a.to_xyz_km().dist_to(&b.to_xyz_km()) * 1000.0;
        assert!(distance_m > 40.0);
        assert!(distance_m < 50.0);
    }

    #[test]
    fn test_bidirectional_conversion() {
        let a = LatLon::new(52.6374196, 13.2054151);
        let b = a.to_xyz_km().to_lat_lon();

        println!("{a:?} -> {b:?}");

        assert!((a.latitude - b.latitude).abs() < 0.0001);
        assert!((a.longitude - b.longitude).abs() < 0.0001);
    }

    #[test]
    fn test_bidirectional_conversion_fuzz() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(42);
        for _ in 0..1000 {
            let a = LatLon::new(
                rng.random_range(-90.0..90.0),
                rng.random_range(-180.0..180.0),
            );
            let xyz = a.to_xyz_km();
            let b = xyz.to_lat_lon();
            println!("{a:?} -> {b:?}");
            assert!((a.latitude - b.latitude).abs() < 0.01);
            assert!((a.longitude - b.longitude).abs() < 0.0001);
        }
    }
}
