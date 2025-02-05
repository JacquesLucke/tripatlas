use std::{fmt::Debug, path::Path, time::Instant};

use anyhow::anyhow;

use crate::csv_parse::{self, ParsedCsv};

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

pub trait FromGtfsStr: Sized {
    fn from_gtfs_str(s: &str) -> Option<Self>;
}

#[derive(Debug, Clone, Default)]
pub enum PickupType {
    #[default]
    Regular,
    NotAvailable,
    MustPhone,
    MustCoordinateWithDriver,
}

impl FromGtfsStr for PickupType {
    fn from_gtfs_str(s: &str) -> Option<Self> {
        match s {
            "0" => Some(PickupType::Regular),
            "1" => Some(PickupType::NotAvailable),
            "2" => Some(PickupType::MustPhone),
            "3" => Some(PickupType::MustCoordinateWithDriver),
            _ => None,
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
}

impl FromGtfsStr for DropOffType {
    fn from_gtfs_str(s: &str) -> Option<Self> {
        match s {
            "0" => Some(DropOffType::Regular),
            "1" => Some(DropOffType::NotAvailable),
            "2" => Some(DropOffType::MustPhone),
            "3" => Some(DropOffType::MustCoordinateWithDriver),
            _ => None,
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
}

impl FromGtfsStr for ContinuousPickupType {
    fn from_gtfs_str(s: &str) -> Option<Self> {
        match s {
            "0" => Some(ContinuousPickupType::Regular),
            "1" => Some(ContinuousPickupType::NotAvailable),
            "2" => Some(ContinuousPickupType::MustPhone),
            "3" => Some(ContinuousPickupType::MustCoordinateWithDriver),
            _ => None,
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
}

impl FromGtfsStr for ContinuousDropOffType {
    fn from_gtfs_str(s: &str) -> Option<Self> {
        match s {
            "0" => Some(ContinuousDropOffType::Regular),
            "1" => Some(ContinuousDropOffType::NotAvailable),
            "2" => Some(ContinuousDropOffType::MustPhone),
            "3" => Some(ContinuousDropOffType::MustCoordinateWithDriver),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum TimePointType {
    Approximate,
    #[default]
    Exact,
}

impl FromGtfsStr for TimePointType {
    fn from_gtfs_str(s: &str) -> Option<Self> {
        match s {
            "0" => Some(TimePointType::Approximate),
            "1" => Some(TimePointType::Exact),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ServiceDayTime {
    seconds: u32,
}

impl FromGtfsStr for f32 {
    fn from_gtfs_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

impl FromGtfsStr for ServiceDayTime {
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
    let stop_times = parse_stop_times(&buffer);
    println!("Detail stop times: {:#?}", parse_times_start.elapsed());

    // println!("{:#?}", stop_times);
}

fn parse_stop_times(buffer: &[u8]) -> anyhow::Result<IndexedGtfsStopTimes> {
    let csv = csv_parse::ParsedCsv::from_buffer(buffer);
    let header_indices = csv.header_indices(buffer);

    let get_required_column_index = |column_name: &str| -> Result<usize, anyhow::Error> {
        Ok(*header_indices
            .get(column_name.as_bytes())
            .ok_or_else(|| anyhow!(format!("Missing column: {}", column_name)))?)
    };

    let column_i_trip_id = get_required_column_index("trip_id")?;
    let column_i_stop_id = get_required_column_index("stop_id")?;
    let column_i_stop_sequence = get_required_column_index("stop_sequence")?;
    let column_i_arrival_time = get_required_column_index("arrival_time")?;
    let column_i_departure_time = get_required_column_index("departure_time")?;

    let mut column_trip_id = vec![];
    let mut column_stop_id = vec![];
    let mut column_stop_sequence = vec![];
    let mut column_arrival_time = vec![];
    let mut column_departure_time = vec![];

    let rows_len = csv.rows_len();
    column_trip_id.reserve(rows_len);
    column_stop_id.reserve(rows_len);
    column_stop_sequence.reserve(rows_len);
    column_arrival_time.reserve(rows_len);
    column_departure_time.reserve(rows_len);

    let get_field_string =
        |row: &csv_parse::ParsedCsvRow, field_i: usize| -> Result<&str, anyhow::Error> {
            if field_i >= row.fields_len() {
                return Err(anyhow!("Row is missing columns"));
            }
            std::str::from_utf8(&row.field(buffer, field_i)).map_err(|_| anyhow!("Invalid UTF-8"))
        };

    // Handle always available columns in one go.
    for row_i in 0..rows_len {
        let row = csv.row(row_i);
        column_trip_id.push(get_field_string(&row, column_i_trip_id)?);
        column_stop_id.push(get_field_string(&row, column_i_stop_id)?);
        column_stop_sequence.push(get_field_string(&row, column_i_stop_sequence)?.parse()?);
        column_arrival_time.push(ServiceDayTime::from_gtfs_str(get_field_string(
            &row,
            column_i_arrival_time,
        )?));
        column_departure_time.push(ServiceDayTime::from_gtfs_str(get_field_string(
            &row,
            column_i_departure_time,
        )?));
    }

    Ok(IndexedGtfsStopTimes {
        trip_id: column_trip_id,
        stop_id: column_stop_id,
        stop_sequence: column_stop_sequence,
        arrival_time: column_arrival_time,
        departure_time: column_departure_time,

        location_group_id: load_optional_string_column("location_group_id", &csv, buffer),
        location_id: load_optional_string_column("location_id", &csv, buffer),
        stop_headsign: load_optional_string_column("stop_headsign", &csv, buffer),
        start_pickup_drop_off_window: load_optional_column_without_default(
            "start_pickup_drop_off_window",
            &csv,
            buffer,
        ),
        end_pickup_drop_off_window: load_optional_column_without_default(
            "end_pickup_drop_off_window",
            &csv,
            buffer,
        ),
        pickup_type: load_optional_column_with_default("pickup_type", &csv, buffer),
        drop_off_type: load_optional_column_with_default("drop_off_type", &csv, buffer),
        continuous_pickup: load_optional_column_with_default("continuous_pickup", &csv, buffer),
        continuous_drop_off: load_optional_column_with_default("continuous_drop_off", &csv, buffer),
        shape_dist_traveled: load_optional_column_without_default(
            "shape_dist_traveled",
            &csv,
            buffer,
        ),
        timepoint: load_optional_column_with_default("timepoint", &csv, buffer),
        pickup_booking_rule_id: load_optional_string_column("pickup_booking_rule_id", &csv, buffer),
        drop_off_booking_rule_id: load_optional_string_column(
            "drop_off_booking_rule_id",
            &csv,
            buffer,
        ),

        ..Default::default()
    })
}

fn load_optional_string_column<'a>(
    column_name: &str,
    csv: &ParsedCsv,
    buffer: &'a [u8],
) -> Option<Vec<&'a str>> {
    let header_indices = csv.header_indices(buffer);
    let column_i = match header_indices.get(column_name.as_bytes()) {
        Some(column_i) => *column_i,
        None => return None,
    };
    let mut column_data = vec![];

    for row_i in 0..csv.rows_len() {
        let row = csv.row(row_i);
        let field = if column_i < row.fields_len() {
            std::str::from_utf8(&row.field(buffer, column_i)).unwrap_or("")
        } else {
            ""
        };
        column_data.push(field);
    }

    Some(column_data)
}

fn load_optional_column_without_default<'a, T: FromGtfsStr>(
    column_name: &str,
    csv: &ParsedCsv,
    buffer: &'a [u8],
) -> Option<Vec<Option<T>>> {
    let header_indices = csv.header_indices(buffer);
    let column_i = match header_indices.get(column_name.as_bytes()) {
        Some(column_i) => *column_i,
        None => return None,
    };
    let mut column_data = vec![];

    for row_i in 0..csv.rows_len() {
        let row = csv.row(row_i);
        let value = if column_i < row.fields_len() {
            if let Ok(field_str) = std::str::from_utf8(&row.field(buffer, column_i)) {
                T::from_gtfs_str(field_str)
            } else {
                None
            }
        } else {
            None
        };
        column_data.push(value);
    }

    Some(column_data)
}

fn load_optional_column_with_default<'a, T: FromGtfsStr + Default>(
    column_name: &str,
    csv: &ParsedCsv,
    buffer: &'a [u8],
) -> Option<Vec<T>> {
    let header_indices = csv.header_indices(buffer);
    let column_i = match header_indices.get(column_name.as_bytes()) {
        Some(column_i) => *column_i,
        None => return None,
    };
    let mut column_data = vec![];

    for row_i in 0..csv.rows_len() {
        let row = csv.row(row_i);
        let value = if column_i < row.fields_len() {
            if let Ok(field_str) = std::str::from_utf8(&row.field(buffer, column_i)) {
                T::from_gtfs_str(field_str)
            } else {
                None
            }
        } else {
            None
        };
        column_data.push(value.unwrap_or_default());
    }

    Some(column_data)
}
