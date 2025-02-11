use std::f32::consts::PI;

use crate::coordinates::{LatLon, LatLonBounds};

#[derive(Debug, Clone, Copy)]
pub struct WebMercatorTile {
    pub zoom: u8,
    pub x: u32,
    pub y: u32,
}

impl WebMercatorTile {
    pub fn new(zoom: u8, x: u32, y: u32) -> Self {
        Self { x, y, zoom }
    }

    pub fn left_longitude(&self) -> f32 {
        tile_x_to_longitude(self.x, self.zoom)
    }

    pub fn right_longitude(&self) -> f32 {
        tile_x_to_longitude(self.x + 1, self.zoom)
    }

    pub fn top_latitude(&self) -> f32 {
        tile_y_to_latitude(self.y, self.zoom)
    }

    pub fn bottom_latitude(&self) -> f32 {
        tile_y_to_latitude(self.y + 1, self.zoom)
    }

    pub fn top_left_lat_lon(&self) -> LatLon {
        LatLon::new(self.top_latitude(), self.left_longitude())
    }

    pub fn bottom_right_lat_lon(&self) -> LatLon {
        LatLon::new(self.bottom_latitude(), self.right_longitude())
    }

    pub fn to_bounds(&self) -> LatLonBounds {
        LatLonBounds::from_corners(self.top_left_lat_lon(), self.bottom_right_lat_lon())
    }
}

fn tile_x_to_longitude(x: u32, zoom: u8) -> f32 {
    let x = x as f32;
    let tile_num = 2_u64.pow(zoom as u32);
    let tile_num_f = tile_num as f32;
    return x / tile_num_f * 360.0 - 180.0;
}

fn tile_y_to_latitude(y: u32, zoom: u8) -> f32 {
    let y = y as f32;
    let tile_num = 2_u64.pow(zoom as u32);
    let tile_num_f = tile_num as f32;
    return ((1.0 - 2.0 * y / tile_num_f) * PI).sinh().atan() * 180.0 / PI;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds() {
        let pos = LatLon::new(52.637641, 13.205084);
        assert!(WebMercatorTile::new(18, 140687, 85830)
            .to_bounds()
            .contains(pos));
        assert!(!WebMercatorTile::new(18, 140688, 85830)
            .to_bounds()
            .contains(pos));
    }
}
