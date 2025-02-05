use std::{fmt::Debug, path::Path, time::Instant};

use rayon::vec;

use crate::csv_parse;

// pub struct IndexedGtfs<'a> {
//     pub stop_times: Vec<IndexedGtfsStopTime<'a>>,
// }

#[derive(Debug, Clone)]
pub struct IndexedGtfsStopTime<'a> {
    pub trip_id: &'a str,
    pub arrival_time: Option<ServiceDayTime>,
    pub departure_time: Option<ServiceDayTime>,
    pub stop_id: &'a str,
    stop_sequence: u32,
    pickup_type: u32,
    drop_off_type: u32,
}

#[derive(Copy, Clone)]
pub struct ServiceDayTime {
    seconds: u32,
}

impl ServiceDayTime {
    pub fn from_str(time: &str) -> Option<Self> {
        let mut parts = time.split(":");
        if let Some(hours) = parts.next() {
            if let Some(minutes) = parts.next() {
                if let Some(seconds) = parts.next() {
                    let total_seconds = hours.parse::<u32>().ok()? * 3600
                        + minutes.parse::<u32>().ok()? * 60
                        + seconds.parse::<u32>().ok()?;
                    return Some(ServiceDayTime {
                        seconds: total_seconds,
                    });
                }
            }
        }
        None
    }
}

impl Debug for ServiceDayTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceDayTime")
            .field(
                "time",
                &format!(
                    "{:02}:{:02}:{:02}",
                    self.seconds / 3600,
                    (self.seconds / 60) % 60,
                    self.seconds % 60
                ),
            )
            .finish()
    }
}

pub fn parse_performance_test() {
    let start_load = Instant::now();
    let path = Path::new("/home/jacques/Documents/gtfs_germany/stop_times.txt");
    let buffer = std::fs::read_to_string(path).unwrap();
    let buffer_bytes = buffer.as_bytes();
    println!("Load buffer: {:?}", start_load.elapsed());

    let start_parse = Instant::now();
    let parsed = csv_parse::ParsedCsv::from_buffer(buffer_bytes);
    println!("Parse buffer: {:?}", start_parse.elapsed());
    println!("Rows: {:?}", parsed.rows_len());

    let header_indices = parsed.header_indices(buffer_bytes);
    let trip_id_i = *header_indices.get("trip_id".as_bytes()).unwrap();
    let arrival_time_i = *header_indices.get("arrival_time".as_bytes()).unwrap();
    let departure_time_i = *header_indices.get("departure_time".as_bytes()).unwrap();
    let stop_id_i = *header_indices.get("stop_id".as_bytes()).unwrap();
    let stop_sequence_i = *header_indices.get("stop_sequence".as_bytes()).unwrap();
    let pickup_type_i = *header_indices.get("pickup_type".as_bytes()).unwrap();
    let drop_off_type_i = *header_indices.get("drop_off_type".as_bytes()).unwrap();

    let detail_parse_start = Instant::now();

    let mut stop_times = vec![];
    stop_times.reserve(parsed.rows_len());
    for row_i in 0..parsed.rows_len() {
        let row = parsed.row(row_i);
        stop_times.push(IndexedGtfsStopTime {
            trip_id: std::str::from_utf8(row.field(buffer_bytes, trip_id_i)).unwrap(),
            arrival_time: ServiceDayTime::from_str(
                std::str::from_utf8(row.field(buffer_bytes, arrival_time_i)).unwrap(),
            ),
            departure_time: ServiceDayTime::from_str(
                std::str::from_utf8(row.field(buffer_bytes, departure_time_i)).unwrap(),
            ),
            stop_id: std::str::from_utf8(row.field(buffer_bytes, stop_id_i)).unwrap(),
            stop_sequence: std::str::from_utf8(row.field(buffer_bytes, stop_sequence_i))
                .unwrap()
                .parse::<u32>()
                .unwrap_or(0),
            pickup_type: std::str::from_utf8(row.field(buffer_bytes, pickup_type_i))
                .unwrap()
                .parse::<u32>()
                .unwrap_or(0),
            drop_off_type: std::str::from_utf8(row.field(buffer_bytes, drop_off_type_i))
                .unwrap()
                .parse::<u32>()
                .unwrap_or(0),
        });
    }

    println!("Detail parse: {:#?}", detail_parse_start.elapsed());
}
