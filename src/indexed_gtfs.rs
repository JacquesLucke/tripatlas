use std::{path::Path, time::Instant};

use crate::csv_parse;

// pub struct IndexedGtfs<'a> {
//     pub stop_times: Vec<IndexedGtfsStopTime<'a>>,
// }

// pub struct IndexedGtfsStopTime<'a> {
//     pub trip_id: &'a str,
// }

pub fn parse_performance_test() {
    let start_load = Instant::now();
    let path = Path::new("/home/jacques/Documents/gtfs_germany/stop_times.txt");
    let buffer = std::fs::read_to_string(path).unwrap();
    println!("Load buffer: {:?}", start_load.elapsed());

    let start_parse = Instant::now();
    let parsed = csv_parse::ParsedCsv::from_buffer(buffer.as_bytes());
    println!("Parse buffer: {:?}", start_parse.elapsed());
    println!("Rows: {:?}", parsed.rows_len());
}
