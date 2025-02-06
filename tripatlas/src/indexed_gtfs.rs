use csvelo::CSVParser;
use rayon::prelude::*;
use std::{fmt::Debug, path::Path, time::Instant};

// GTFS Reference: https://gtfs.org/documentation/schedule/reference/

// pub struct IndexedGtfs<'a> {
//     pub stop_times: Vec<IndexedGtfsStopTime<'a>>,
// }

#[derive(CSVParser, Debug, Clone, Default)]
pub struct StopTimes<'a> {
    pub trip_id: Vec<&'a str>,
    pub stop_id: Vec<&'a str>,
    pub stop_sequence: Vec<u32>,
    pub arrival_time: Vec<OptionalServiceDayTime>,
    pub departure_time: Vec<OptionalServiceDayTime>,

    pub location_group_id: Option<Vec<&'a str>>,
    pub location_id: Option<Vec<&'a str>>,
    pub stop_headsign: Option<Vec<&'a str>>,
    pub start_pickup_drop_off_window: Option<Vec<OptionalServiceDayTime>>,
    pub end_pickup_drop_off_window: Option<Vec<OptionalServiceDayTime>>,
    pub pickup_type: Option<Vec<PickupType>>,
    pub drop_off_type: Option<Vec<DropOffType>>,
    pub continuous_pickup: Option<Vec<ContinuousPickupType>>,
    pub continuous_drop_off: Option<Vec<ContinuousDropOffType>>,
    pub shape_dist_traveled: Option<Vec<OptionalF32>>,
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

impl<'a> csvelo::ParseCsvField<'a> for PickupType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        Ok(PickupType::from_gtfs_str(
            std::str::from_utf8(buffer).map_err(|_| ())?,
        ))
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

impl<'a> csvelo::ParseCsvField<'a> for DropOffType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        Ok(DropOffType::from_gtfs_str(
            std::str::from_utf8(buffer).map_err(|_| ())?,
        ))
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

impl<'a> csvelo::ParseCsvField<'a> for ContinuousPickupType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        Ok(ContinuousPickupType::from_gtfs_str(
            std::str::from_utf8(buffer).map_err(|_| ())?,
        ))
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

impl<'a> csvelo::ParseCsvField<'a> for ContinuousDropOffType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        Ok(ContinuousDropOffType::from_gtfs_str(
            std::str::from_utf8(buffer).map_err(|_| ())?,
        ))
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

impl<'a> csvelo::ParseCsvField<'a> for TimePointType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        Ok(TimePointType::from_gtfs_str(
            std::str::from_utf8(buffer).map_err(|_| ())?,
        ))
    }
}

#[derive(Copy, Clone)]
pub struct ServiceDayTime {
    seconds: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct OptionalServiceDayTime(Option<ServiceDayTime>);

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

#[derive(Debug, Copy, Clone)]
pub struct OptionalF32(Option<f32>);

impl<'a> csvelo::ParseCsvField<'a> for OptionalF32 {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        let s = std::str::from_utf8(buffer).map_err(|_| ())?;
        let f = s.parse::<f32>().map_err(|_| ()).ok();
        Ok(OptionalF32(f))
    }
}

impl<'a> csvelo::ParseCsvField<'a> for OptionalServiceDayTime {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        let s = std::str::from_utf8(buffer).map_err(|_| ())?;
        let time = ServiceDayTime::from_gtfs_str(s);
        Ok(OptionalServiceDayTime(time))
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
    let file = std::fs::File::open(path).unwrap();
    let mmap = unsafe { memmap2::Mmap::map(&file) }.unwrap();
    let buffer = &mmap[..];
    println!("Load buffer: {:?}", start_load.elapsed());

    let parse_times_start = Instant::now();
    let stop_times: StopTimes = StopTimes::from_csv_buffer(&buffer).unwrap();
    println!("Detail stop times: {:#?}", parse_times_start.elapsed());

    println!("{:#?}", stop_times.stop_id.len());
}
