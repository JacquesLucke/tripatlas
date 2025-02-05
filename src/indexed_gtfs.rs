use std::{fmt::Debug, path::Path, time::Instant};

use anyhow::anyhow;

use crate::csv_parse::{self};

// GTFS Reference: https://gtfs.org/documentation/schedule/reference/

// pub struct IndexedGtfs<'a> {
//     pub stop_times: Vec<IndexedGtfsStopTime<'a>>,
// }

#[derive(Debug, Clone, Default)]
pub struct IndexedGtfsStopTimes<'a> {
    pub trip_id: Vec<&'a str>,
    pub stop_id: Vec<&'a str>,
    pub stop_sequence: Vec<u32>,
    pub arrival_time: Vec<Option<ServiceDayTime>>,
    pub departure_time: Vec<Option<ServiceDayTime>>,

    pub location_group_id: Option<Vec<&'a str>>,
    pub location_id: Option<Vec<&'a str>>,
    pub stop_headsign: Option<Vec<&'a str>>,
    pub start_pickup_drop_off_window: Option<Vec<Option<ServiceDayTime>>>,
    pub end_pickup_drop_off_window: Option<Vec<Option<ServiceDayTime>>>,
    pub pickup_type: Option<Vec<PickupType>>,
    pub drop_off_type: Option<Vec<DropOffType>>,
    pub continuous_pickup: Option<Vec<ContinuousPickupType>>,
    pub continuous_drop_off: Option<Vec<ContinuousDropOffType>>,
    pub shape_dist_traveled: Option<Vec<Option<f32>>>,
    pub timepoint: Option<Vec<TimePointType>>,
    pub pickup_booking_rule_id: Option<Vec<&'a str>>,
    pub drop_off_booking_rule_id: Option<Vec<&'a str>>,
}

#[derive(Debug, Clone, Default)]
pub enum PickupType {
    #[default]
    Regular,
    NotAvailable,
    MustPhone,
    MustCoordinateWithDriver,
    Unknown,
}

impl PickupType {
    fn from_gtfs_str(s: &str) -> Self {
        match s {
            "0" => PickupType::Regular,
            "1" => PickupType::NotAvailable,
            "2" => PickupType::MustPhone,
            "3" => PickupType::MustCoordinateWithDriver,
            _ => PickupType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum DropOffType {
    #[default]
    Regular,
    NotAvailable,
    MustPhone,
    MustCoordinateWithDriver,
    Unknown,
}

impl DropOffType {
    fn from_gtfs_str(s: &str) -> Self {
        match s {
            "0" => DropOffType::Regular,
            "1" => DropOffType::NotAvailable,
            "2" => DropOffType::MustPhone,
            "3" => DropOffType::MustCoordinateWithDriver,
            _ => DropOffType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum ContinuousPickupType {
    #[default]
    Regular,
    NotAvailable,
    MustPhone,
    MustCoordinateWithDriver,
    Unknown,
}

impl ContinuousPickupType {
    fn from_gtfs_str(s: &str) -> Self {
        match s {
            "0" => ContinuousPickupType::Regular,
            "1" => ContinuousPickupType::NotAvailable,
            "2" => ContinuousPickupType::MustPhone,
            "3" => ContinuousPickupType::MustCoordinateWithDriver,
            _ => ContinuousPickupType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum ContinuousDropOffType {
    #[default]
    Regular,
    NotAvailable,
    MustPhone,
    MustCoordinateWithDriver,
    Unknown,
}

impl ContinuousDropOffType {
    fn from_gtfs_str(s: &str) -> Self {
        match s {
            "0" => ContinuousDropOffType::Regular,
            "1" => ContinuousDropOffType::NotAvailable,
            "2" => ContinuousDropOffType::MustPhone,
            "3" => ContinuousDropOffType::MustCoordinateWithDriver,
            _ => ContinuousDropOffType::Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum TimePointType {
    Approximate,
    #[default]
    Exact,
    Unknown,
}

impl TimePointType {
    fn from_gtfs_str(s: &str) -> Self {
        match s {
            "0" => TimePointType::Approximate,
            "1" => TimePointType::Exact,
            _ => TimePointType::Unknown,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ServiceDayTime {
    seconds: u32,
}

impl ServiceDayTime {
    fn from_gtfs_str(time: &str) -> Option<Self> {
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
    let buffer = std::fs::read(path).unwrap();
    println!("Load buffer: {:?}", start_load.elapsed());

    let parse_times_start = Instant::now();

    let _stop_times = parse_stop_times(&buffer);

    println!("Detail stop times: {:#?}", parse_times_start.elapsed());

    // println!("{:#?}", stop_times);
}

fn parse_stop_times(buffer: &[u8]) -> anyhow::Result<IndexedGtfsStopTimes> {
    let sections = csv_parse::split_header_and_data(&buffer);
    let column_titles = csv_parse::parse_header_row_str(sections.header)?;
    let header = parse_stop_times_header(&column_titles)?;

    let data_chunks = csv_parse::split_csv_buffer_into_line_aligned_chunks(sections.data);
    let mut parsed_chunks = vec![];
    for chunk in data_chunks {
        let rows = csv_parse::CsvRows::from_buffer(chunk);
        parsed_chunks.push(parse_stop_times_chunk(&header, &rows)?);
    }
    merge_stop_time_chunks(&header, parsed_chunks)
}

#[derive(Debug)]
struct StopTimesHeader {
    pub trip_id: usize,
    pub stop_id: usize,
    pub stop_sequence: usize,
    pub arrival_time: usize,
    pub departure_time: usize,

    pub location_group_id: Option<usize>,
    pub location_id: Option<usize>,
    pub stop_headsign: Option<usize>,
    pub start_pickup_drop_off_window: Option<usize>,
    pub end_pickup_drop_off_window: Option<usize>,
    pub pickup_type: Option<usize>,
    pub drop_off_type: Option<usize>,
    pub continuous_pickup: Option<usize>,
    pub continuous_drop_off: Option<usize>,
    pub shape_dist_traveled: Option<usize>,
    pub timepoint: Option<usize>,
    pub pickup_booking_rule_id: Option<usize>,
    pub drop_off_booking_rule_id: Option<usize>,
}
struct StopTimesChunk<'b> {
    pub trip_id: Vec<&'b str>,
    pub stop_id: Vec<&'b str>,
    pub stop_sequence: Vec<u32>,
    pub arrival_time: Vec<Option<ServiceDayTime>>,
    pub departure_time: Vec<Option<ServiceDayTime>>,

    pub location_group_id: Option<Vec<&'b str>>,
    pub location_id: Option<Vec<&'b str>>,
    pub stop_headsign: Option<Vec<&'b str>>,
    pub start_pickup_drop_off_window: Option<Vec<Option<ServiceDayTime>>>,
    pub end_pickup_drop_off_window: Option<Vec<Option<ServiceDayTime>>>,
    pub pickup_type: Option<Vec<PickupType>>,
    pub drop_off_type: Option<Vec<DropOffType>>,
    pub continuous_pickup: Option<Vec<ContinuousPickupType>>,
    pub continuous_drop_off: Option<Vec<ContinuousDropOffType>>,
    pub shape_dist_traveled: Option<Vec<Option<f32>>>,
    pub timepoint: Option<Vec<TimePointType>>,
    pub pickup_booking_rule_id: Option<Vec<&'b str>>,
    pub drop_off_booking_rule_id: Option<Vec<&'b str>>,
}

fn parse_stop_times_header(column_titles: &[&str]) -> anyhow::Result<StopTimesHeader> {
    Ok(StopTimesHeader {
        trip_id: get_required_column_index(column_titles, "trip_id")?,
        stop_id: get_required_column_index(column_titles, "stop_id")?,
        stop_sequence: get_required_column_index(column_titles, "stop_sequence")?,
        arrival_time: get_required_column_index(column_titles, "arrival_time")?,
        departure_time: get_required_column_index(column_titles, "departure_time")?,
        location_group_id: get_optional_column_index(column_titles, "location_group_id"),
        location_id: get_optional_column_index(column_titles, "location_id"),
        stop_headsign: get_optional_column_index(column_titles, "stop_headsign"),
        start_pickup_drop_off_window: get_optional_column_index(
            column_titles,
            "start_pickup_drop_off_window",
        ),
        end_pickup_drop_off_window: get_optional_column_index(
            column_titles,
            "end_pickup_drop_off_window",
        ),
        pickup_type: get_optional_column_index(column_titles, "pickup_type"),
        drop_off_type: get_optional_column_index(column_titles, "drop_off_type"),
        continuous_pickup: get_optional_column_index(column_titles, "continuous_pickup"),
        continuous_drop_off: get_optional_column_index(column_titles, "continuous_drop_off"),
        shape_dist_traveled: get_optional_column_index(column_titles, "shape_dist_traveled"),
        timepoint: get_optional_column_index(column_titles, "timepoint"),
        pickup_booking_rule_id: get_optional_column_index(column_titles, "pickup_booking_rule_id"),
        drop_off_booking_rule_id: get_optional_column_index(
            column_titles,
            "drop_off_booking_rule_id",
        ),
    })
}

fn parse_stop_times_chunk<'a>(
    header: &StopTimesHeader,
    rows: &csv_parse::CsvRows<'a>,
) -> anyhow::Result<StopTimesChunk<'a>> {
    Ok(StopTimesChunk {
        trip_id: load_column(rows, header.trip_id, |s| Ok(s))?,
        stop_id: load_column(rows, header.stop_id, |s| Ok(s))?,
        stop_sequence: load_column(rows, header.stop_sequence, |s| Ok(s.parse()?))?,
        arrival_time: load_column(rows, header.arrival_time, |s| {
            Ok(ServiceDayTime::from_gtfs_str(s))
        })?,
        departure_time: load_column(rows, header.departure_time, |s| {
            Ok(ServiceDayTime::from_gtfs_str(s))
        })?,
        location_group_id: match header.location_group_id {
            Some(i) => Some(load_column(rows, i, |s| Ok(s))?),
            None => None,
        },
        location_id: match header.location_id {
            Some(i) => Some(load_column(rows, i, |s| Ok(s))?),
            None => None,
        },
        stop_headsign: match header.stop_headsign {
            Some(i) => Some(load_column(rows, i, |s| Ok(s))?),
            None => None,
        },
        start_pickup_drop_off_window: match header.start_pickup_drop_off_window {
            Some(i) => Some(load_column(rows, i, |s| {
                Ok(ServiceDayTime::from_gtfs_str(s))
            })?),
            None => None,
        },
        end_pickup_drop_off_window: match header.end_pickup_drop_off_window {
            Some(i) => Some(load_column(rows, i, |s| {
                Ok(ServiceDayTime::from_gtfs_str(s))
            })?),
            None => None,
        },
        pickup_type: match header.pickup_type {
            Some(i) => Some(load_column(rows, i, |s| Ok(PickupType::from_gtfs_str(s)))?),
            None => None,
        },
        drop_off_type: match header.drop_off_type {
            Some(i) => Some(load_column(rows, i, |s| Ok(DropOffType::from_gtfs_str(s)))?),
            None => None,
        },
        continuous_pickup: match header.continuous_pickup {
            Some(i) => Some(load_column(rows, i, |s| {
                Ok(ContinuousPickupType::from_gtfs_str(s))
            })?),
            None => None,
        },
        continuous_drop_off: match header.continuous_drop_off {
            Some(i) => Some(load_column(rows, i, |s| {
                Ok(ContinuousDropOffType::from_gtfs_str(s))
            })?),
            None => None,
        },
        shape_dist_traveled: match header.shape_dist_traveled {
            Some(i) => Some(load_column(rows, i, |s| Ok(s.parse().ok()))?),
            None => None,
        },
        timepoint: match header.timepoint {
            Some(i) => Some(load_column(rows, i, |s| {
                Ok(TimePointType::from_gtfs_str(s))
            })?),
            None => None,
        },
        pickup_booking_rule_id: match header.pickup_booking_rule_id {
            Some(i) => Some(load_column(rows, i, |s| Ok(s))?),
            None => None,
        },
        drop_off_booking_rule_id: match header.drop_off_booking_rule_id {
            Some(i) => Some(load_column(rows, i, |s| Ok(s))?),
            None => None,
        },
    })
}

fn merge_stop_time_chunks<'b>(
    header: &StopTimesHeader,
    chunks: Vec<StopTimesChunk<'b>>,
) -> anyhow::Result<IndexedGtfsStopTimes<'b>> {
    Ok(IndexedGtfsStopTimes {
        trip_id: chunks.iter().flat_map(|c| c.trip_id.clone()).collect(),
        stop_id: chunks.iter().flat_map(|c| c.stop_id.clone()).collect(),
        stop_sequence: chunks
            .iter()
            .flat_map(|c| c.stop_sequence.clone())
            .collect(),
        arrival_time: chunks.iter().flat_map(|c| c.arrival_time.clone()).collect(),
        departure_time: chunks
            .iter()
            .flat_map(|c| c.departure_time.clone())
            .collect(),
        location_group_id: match header.location_group_id {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.location_group_id.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        location_id: match header.location_id {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.location_id.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        stop_headsign: match header.stop_headsign {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.stop_headsign.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        start_pickup_drop_off_window: match header.start_pickup_drop_off_window {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.start_pickup_drop_off_window.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        end_pickup_drop_off_window: match header.end_pickup_drop_off_window {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.end_pickup_drop_off_window.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        pickup_type: match header.pickup_type {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.pickup_type.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        drop_off_type: match header.drop_off_type {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.drop_off_type.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        continuous_pickup: match header.continuous_pickup {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.continuous_pickup.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        continuous_drop_off: match header.continuous_drop_off {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.continuous_drop_off.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        shape_dist_traveled: match header.shape_dist_traveled {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.shape_dist_traveled.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        timepoint: match header.timepoint {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.timepoint.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        pickup_booking_rule_id: match header.pickup_booking_rule_id {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.pickup_booking_rule_id.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
        drop_off_booking_rule_id: match header.drop_off_booking_rule_id {
            Some(_) => Some(
                chunks
                    .iter()
                    .flat_map(|c| c.drop_off_booking_rule_id.clone().unwrap())
                    .collect(),
            ),
            None => None,
        },
    })
}

fn get_required_column_index(
    column_titles: &[&str],
    column_name: &str,
) -> Result<usize, anyhow::Error> {
    column_titles
        .iter()
        .position(|&h| h == column_name)
        .ok_or_else(|| anyhow!(format!("Missing column: {}", column_name)))
}

fn get_optional_column_index(column_titles: &[&str], column_name: &str) -> Option<usize> {
    column_titles.iter().position(|&h| h == column_name)
}

fn load_column<'a, T>(
    rows: &csv_parse::CsvRows<'a>,
    column_i: usize,
    f: impl Fn(&'a str) -> anyhow::Result<T>,
) -> anyhow::Result<Vec<T>> {
    let mut data = vec![];
    data.reserve(rows.len());
    for row in rows.iter() {
        let column_buffer = row.column(column_i);
        let column_buffer = column_buffer.unwrap_or(b"");
        let column_str = std::str::from_utf8(column_buffer)?;
        let value = f(column_str)?;
        data.push(value);
    }
    Ok(data)
}
