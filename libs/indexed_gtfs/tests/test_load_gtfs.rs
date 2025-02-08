use std::path::Path;

use indexed_gtfs::*;

#[test]
fn test_load_gtfs_dummy() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("testdata")
        .join("gtfs_dummy");
    let buffers = GtfsBuffers::from_dir(&path, &GtfsFilter::all());
    let gtfs = Gtfs::from_buffers(buffers.to_slices()).unwrap();
    assert_eq!(gtfs.stops.len, 2);

    let stops = gtfs.stops.data.unwrap();

    assert_eq!(stops.stop_id.unwrap(), vec!["1", "2"]);
    assert_eq!(
        stops.stop_name.unwrap(),
        vec!["My Station", "Another Station"]
    );
    assert_eq!(
        stops
            .stop_lat
            .unwrap()
            .iter()
            .map(|v| v.0.unwrap() as i32)
            .collect::<Vec<i32>>(),
        vec![42, 10]
    );
    assert_eq!(
        stops
            .stop_lon
            .unwrap()
            .iter()
            .map(|v| v.0.unwrap() as i32)
            .collect::<Vec<i32>>(),
        vec![24, 11]
    );
    assert_eq!(
        stops.location_type.unwrap(),
        vec![LocationType::Station, LocationType::Station]
    );
}
