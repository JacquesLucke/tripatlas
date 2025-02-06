use csvelo::CSVParser;
use num_format::ToFormattedString;
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

impl<'a> csvelo::ParseCsvField<'a> for PickupType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        if buffer.len() != 1 {
            return Ok(PickupType::Unknown);
        }
        match buffer[0] {
            b'0' => Ok(PickupType::Regular),
            b'1' => Ok(PickupType::NotAvailable),
            b'2' => Ok(PickupType::MustPhone),
            b'3' => Ok(PickupType::MustCoordinateWithDriver),
            _ => Ok(PickupType::Unknown),
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

impl<'a> csvelo::ParseCsvField<'a> for DropOffType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        if buffer.len() != 1 {
            return Ok(DropOffType::Unknown);
        }
        match buffer[0] {
            b'0' => Ok(DropOffType::Regular),
            b'1' => Ok(DropOffType::NotAvailable),
            b'2' => Ok(DropOffType::MustPhone),
            b'3' => Ok(DropOffType::MustCoordinateWithDriver),
            _ => Ok(DropOffType::Unknown),
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

impl<'a> csvelo::ParseCsvField<'a> for ContinuousPickupType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        if buffer.len() != 1 {
            return Ok(ContinuousPickupType::Unknown);
        }
        match buffer[0] {
            b'0' => Ok(ContinuousPickupType::Regular),
            b'1' => Ok(ContinuousPickupType::NotAvailable),
            b'2' => Ok(ContinuousPickupType::MustPhone),
            b'3' => Ok(ContinuousPickupType::MustCoordinateWithDriver),
            _ => Ok(ContinuousPickupType::Unknown),
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

impl<'a> csvelo::ParseCsvField<'a> for ContinuousDropOffType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        if buffer.len() != 1 {
            return Ok(ContinuousDropOffType::Unknown);
        }
        match buffer[0] {
            b'0' => Ok(ContinuousDropOffType::Regular),
            b'1' => Ok(ContinuousDropOffType::NotAvailable),
            b'2' => Ok(ContinuousDropOffType::MustPhone),
            b'3' => Ok(ContinuousDropOffType::MustCoordinateWithDriver),
            _ => Ok(ContinuousDropOffType::Unknown),
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

impl<'a> csvelo::ParseCsvField<'a> for TimePointType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        if buffer.len() != 1 {
            return Ok(TimePointType::Unknown);
        }
        match buffer[0] {
            b'0' => Ok(TimePointType::Approximate),
            b'1' => Ok(TimePointType::Exact),
            _ => Ok(TimePointType::Unknown),
        }
    }
}

#[derive(Copy, Clone)]
pub struct ServiceDayTime {
    seconds: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct OptionalServiceDayTime(Option<ServiceDayTime>);

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

fn parse_two_digit_int(buffer: &[u8]) -> u8 {
    return buffer[0].wrapping_sub(b'0') * 10 + buffer[1].wrapping_sub(b'0');
}

fn parse_hh_mm_ss_to_seconds_fast(buffer: &[u8]) -> Result<u32, ()> {
    if buffer.len() >= 8 && buffer[2] == b':' && buffer[5] == b':' {
        let h = parse_two_digit_int(&buffer[0..2]) as u32;
        let m = parse_two_digit_int(&buffer[3..5]) as u32;
        let s = parse_two_digit_int(&buffer[6..8]) as u32;
        return Ok(h * 3600 + m * 60 + s);
    }
    Err(())
}

impl<'a> csvelo::ParseCsvField<'a> for OptionalServiceDayTime {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        if let Ok(seconds) = parse_hh_mm_ss_to_seconds_fast(buffer.trim_ascii()) {
            return Ok(OptionalServiceDayTime(Some(ServiceDayTime { seconds })));
        }
        Ok(OptionalServiceDayTime(None))
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
    let gtfs_dir = Path::new("/home/jacques/Documents/gtfs_germany");

    {
        let stop_times_timer = Instant::now();
        let stop_times_path = gtfs_dir.join("stop_times.txt");
        let stop_times_file = std::fs::File::open(stop_times_path).unwrap();
        let stop_times_mmap = unsafe { memmap2::Mmap::map(&stop_times_file) }.unwrap();
        let stop_times: StopTimes = parse_stop_times(&stop_times_mmap[..]).unwrap();
        println!(
            "Stop Times: {:?}, found: {}",
            stop_times_timer.elapsed(),
            stop_times
                .stop_id
                .len()
                .to_formatted_string(&num_format::Locale::en)
        );
    }
}

fn parse_stop_times<'a>(buffer: &'a [u8]) -> Result<StopTimes<'a>, ()> {
    StopTimes::from_csv_buffer(&buffer)
}
