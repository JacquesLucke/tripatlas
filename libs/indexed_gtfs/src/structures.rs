use csvelo::CSVParser;
use rayon::prelude::*;
use std::fmt::Debug;

// GTFS Reference: https://gtfs.org/documentation/schedule/reference/

pub struct Gtfs<'a> {
    pub stop_times: File<StopTimes<'a>>,
    pub stops: File<Stops<'a>>,
    pub trips: File<Trips<'a>>,
    pub routes: File<Routes<'a>>,
    pub calendars: File<Calendar<'a>>,
    pub calendar_dates: File<CalendarDates<'a>>,
    pub agencies: File<Agencies<'a>>,
    pub feed_infos: File<FeedInfos<'a>>,
    pub attributions: File<Attributions<'a>>,
}

impl Debug for Gtfs<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gtfs")
            .field("stop_times", &self.stop_times.len)
            .field("stops", &self.stops.len)
            .field("trips", &self.trips.len)
            .field("routes", &self.routes.len)
            .field("calendar", &self.calendars.len)
            .field("calendar_dates", &self.calendar_dates.len)
            .field("agencies", &self.agencies.len)
            .field("feed_infos", &self.feed_infos.len)
            .field("attributions", &self.attributions.len)
            .finish()
    }
}

#[derive(Debug)]
pub struct File<T> {
    pub len: usize,
    pub data: Option<T>,
}

#[derive(CSVParser, Debug, Clone, Default)]
pub struct StopTimes<'a> {
    pub trip_id: Option<Vec<&'a str>>,
    pub stop_id: Option<Vec<&'a str>>,
    pub stop_sequence: Option<Vec<u32>>,
    pub arrival_time: Option<Vec<OptionalServiceDayTime>>,
    pub departure_time: Option<Vec<OptionalServiceDayTime>>,

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

#[derive(CSVParser, Debug, Clone, Default)]
pub struct Stops<'a> {
    pub stop_id: Option<Vec<&'a str>>,
    pub stop_code: Option<Vec<&'a str>>,
    pub stop_name: Option<Vec<&'a str>>,
    pub tts_stop_name: Option<Vec<&'a str>>,
    pub stop_desc: Option<Vec<&'a str>>,
    pub stop_lat: Option<Vec<OptionalF32>>,
    pub stop_lon: Option<Vec<OptionalF32>>,
    pub zone_id: Option<Vec<&'a str>>,
    pub stop_url: Option<Vec<&'a str>>,
    pub location_type: Option<Vec<LocationType>>,
    pub parent_station: Option<Vec<&'a str>>,
    pub stop_timezone: Option<Vec<&'a str>>,
    pub wheelchair_boarding: Option<Vec<WheelchairBoarding>>,
    pub level_id: Option<Vec<&'a str>>,
    pub platform_code: Option<Vec<&'a str>>,
}

#[derive(CSVParser, Debug, Clone, Default)]
pub struct Trips<'a> {
    pub route_id: Option<Vec<&'a str>>,
    pub service_id: Option<Vec<&'a str>>,
    pub trip_id: Option<Vec<&'a str>>,
    pub trip_headsign: Option<Vec<&'a str>>,
    pub trip_short_name: Option<Vec<&'a str>>,
    pub direction_id: Option<Vec<DirectionId>>,
    pub block_id: Option<Vec<&'a str>>,
    pub shape_id: Option<Vec<&'a str>>,
    pub wheelchair_accessible: Option<Vec<WheelchairAccessible>>,
    pub bikes_allowed: Option<Vec<BikesAllowed>>,
}

#[derive(CSVParser, Debug, Clone, Default)]
pub struct Routes<'a> {
    pub route_id: Option<Vec<&'a str>>,
    pub agency_id: Option<Vec<&'a str>>,
    pub route_short_name: Option<Vec<&'a str>>,
    pub route_long_name: Option<Vec<&'a str>>,
    pub route_desc: Option<Vec<&'a str>>,
    pub route_type: Option<Vec<RouteType>>,
    pub route_url: Option<Vec<&'a str>>,
    pub route_color: Option<Vec<Color>>,
    pub route_text_color: Option<Vec<Color>>,
    pub route_sort_order: Option<Vec<u32>>,
    pub continuous_pickup: Option<Vec<ContinuousPickupType>>,
    pub continuous_drop_off: Option<Vec<ContinuousDropOffType>>,
    pub network_id: Option<Vec<&'a str>>,
}

#[derive(CSVParser, Debug, Clone, Default)]
pub struct Calendar<'a> {
    pub service_id: Option<Vec<&'a str>>,
    pub monday: Option<Vec<ServiceAvailable>>,
    pub tuesday: Option<Vec<ServiceAvailable>>,
    pub wednesday: Option<Vec<ServiceAvailable>>,
    pub thursday: Option<Vec<ServiceAvailable>>,
    pub friday: Option<Vec<ServiceAvailable>>,
    pub saturday: Option<Vec<ServiceAvailable>>,
    pub sunday: Option<Vec<ServiceAvailable>>,
    pub start_date: Option<Vec<Date>>,
    pub end_date: Option<Vec<Date>>,
}

#[derive(CSVParser, Debug, Clone, Default)]
pub struct CalendarDates<'a> {
    pub service_id: Option<Vec<&'a str>>,
    pub date: Option<Vec<Date>>,
    pub exception_type: Option<Vec<ExceptionType>>,
}

#[derive(CSVParser, Debug, Clone, Default)]
pub struct Agencies<'a> {
    pub agency_id: Option<Vec<&'a str>>,
    pub agency_name: Option<Vec<&'a str>>,
    pub agency_url: Option<Vec<&'a str>>,
    pub agency_timezone: Option<Vec<&'a str>>,
    pub agency_lang: Option<Vec<&'a str>>,
    pub agency_phone: Option<Vec<&'a str>>,
    pub agency_fare_url: Option<Vec<&'a str>>,
    pub agency_email: Option<Vec<&'a str>>,
}

#[derive(CSVParser, Debug, Clone, Default)]
pub struct FeedInfos<'a> {
    pub feed_publisher_name: Option<Vec<&'a str>>,
    pub feed_publisher_url: Option<Vec<&'a str>>,
    pub feed_lang: Option<Vec<&'a str>>,
    pub default_lang: Option<Vec<&'a str>>,
    pub feed_start_date: Option<Vec<Date>>,
    pub feed_end_date: Option<Vec<Date>>,
    pub feed_version: Option<Vec<&'a str>>,
    pub feed_contact_email: Option<Vec<&'a str>>,
    pub feed_contact_url: Option<Vec<&'a str>>,
}

#[derive(CSVParser, Debug, Clone, Default)]
pub struct Attributions<'a> {
    attribution_id: Option<Vec<&'a str>>,
    agency_id: Option<Vec<&'a str>>,
    route_id: Option<Vec<&'a str>>,
    trip_id: Option<Vec<&'a str>>,
    organization_name: Option<Vec<&'a str>>,
    is_producer: Option<Vec<YesOrNo>>,
    is_operator: Option<Vec<YesOrNo>>,
    is_authority: Option<Vec<YesOrNo>>,
    attribution_url: Option<Vec<&'a str>>,
    attribution_email: Option<Vec<&'a str>>,
    attribution_phone: Option<Vec<&'a str>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
        match buffer.trim_ascii() {
            b"" | b"0" => Ok(PickupType::Regular),
            b"1" => Ok(PickupType::NotAvailable),
            b"2" => Ok(PickupType::MustPhone),
            b"3" => Ok(PickupType::MustCoordinateWithDriver),
            _ => Ok(PickupType::Unknown),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
        match buffer.trim_ascii() {
            b"" | b"0" => Ok(DropOffType::Regular),
            b"1" => Ok(DropOffType::NotAvailable),
            b"2" => Ok(DropOffType::MustPhone),
            b"3" => Ok(DropOffType::MustCoordinateWithDriver),
            _ => Ok(DropOffType::Unknown),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
        match buffer.trim_ascii() {
            b"" | b"0" => Ok(ContinuousPickupType::Regular),
            b"1" => Ok(ContinuousPickupType::NotAvailable),
            b"2" => Ok(ContinuousPickupType::MustPhone),
            b"3" => Ok(ContinuousPickupType::MustCoordinateWithDriver),
            _ => Ok(ContinuousPickupType::Unknown),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
        match buffer.trim_ascii() {
            b"" | b"0" => Ok(ContinuousDropOffType::Regular),
            b"1" => Ok(ContinuousDropOffType::NotAvailable),
            b"2" => Ok(ContinuousDropOffType::MustPhone),
            b"3" => Ok(ContinuousDropOffType::MustCoordinateWithDriver),
            _ => Ok(ContinuousDropOffType::Unknown),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
        match buffer.trim_ascii() {
            b"0" => Ok(TimePointType::Approximate),
            b"" | b"1" => Ok(TimePointType::Exact),
            _ => Ok(TimePointType::Unknown),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LocationType {
    #[default]
    Stop,
    Station,
    Entrance,
    Generic,
    BoardingArea,
    Unknown,
}

impl<'a> csvelo::ParseCsvField<'a> for LocationType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        match buffer.trim_ascii() {
            b"" | b"0" => Ok(LocationType::Stop),
            b"1" => Ok(LocationType::Station),
            b"2" => Ok(LocationType::Entrance),
            b"3" => Ok(LocationType::Generic),
            b"4" => Ok(LocationType::BoardingArea),
            _ => Ok(LocationType::Unknown),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WheelchairBoarding {
    #[default]
    NoInfoOrSeeParent,
    SomeAccessibility,
    NoAccessibility,
    Unknown,
}

impl<'a> csvelo::ParseCsvField<'a> for WheelchairBoarding {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        match buffer.trim_ascii() {
            b"" | b"0" => Ok(WheelchairBoarding::NoInfoOrSeeParent),
            b"1" => Ok(WheelchairBoarding::SomeAccessibility),
            b"2" => Ok(WheelchairBoarding::NoAccessibility),
            _ => Ok(WheelchairBoarding::Unknown),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectionId {
    Outbound,
    Inbound,
    Unknown,
}

impl<'a> csvelo::ParseCsvField<'a> for DirectionId {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        match buffer.trim_ascii() {
            b"0" => Ok(DirectionId::Outbound),
            b"1" => Ok(DirectionId::Inbound),
            _ => Ok(DirectionId::Unknown),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WheelchairAccessible {
    #[default]
    NoInfo,
    AtLeastOne,
    No,
    Unknown,
}

impl<'a> csvelo::ParseCsvField<'a> for WheelchairAccessible {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        match buffer.trim_ascii() {
            b"" | b"0" => Ok(WheelchairAccessible::NoInfo),
            b"1" => Ok(WheelchairAccessible::AtLeastOne),
            b"2" => Ok(WheelchairAccessible::No),
            _ => Ok(WheelchairAccessible::Unknown),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum BikesAllowed {
    #[default]
    NoInfo,
    AtLeastOne,
    No,
    Unknown,
}

impl<'a> csvelo::ParseCsvField<'a> for BikesAllowed {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        match buffer.trim_ascii() {
            b"" | b"0" => Ok(BikesAllowed::NoInfo),
            b"1" => Ok(BikesAllowed::AtLeastOne),
            b"2" => Ok(BikesAllowed::No),
            _ => Ok(BikesAllowed::Unknown),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum RouteType {
    #[default]
    Tram,
    Subway,
    Rail,
    Bus,
    Ferry,
    CableTram,
    AerialLift,
    Funicular,
    Trolleybus,
    Monorail,
    Unknown,
}

impl<'a> csvelo::ParseCsvField<'a> for RouteType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        match buffer.trim_ascii() {
            b"0" => Ok(RouteType::Tram),
            b"1" => Ok(RouteType::Subway),
            b"2" => Ok(RouteType::Rail),
            b"3" => Ok(RouteType::Bus),
            b"4" => Ok(RouteType::Ferry),
            b"5" => Ok(RouteType::CableTram),
            b"6" => Ok(RouteType::AerialLift),
            b"7" => Ok(RouteType::Funicular),
            b"11" => Ok(RouteType::Trolleybus),
            b"12" => Ok(RouteType::Monorail),
            _ => Ok(RouteType::Unknown),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl<'a> csvelo::ParseCsvField<'a> for Color {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        let buffer = buffer.trim_ascii();
        if buffer.len() != 6 {
            return Err(());
        }
        let r = hex_char_to_number(buffer[0]) * 16 + hex_char_to_number(buffer[1]);
        let g = hex_char_to_number(buffer[2]) * 16 + hex_char_to_number(buffer[3]);
        let b = hex_char_to_number(buffer[4]) * 16 + hex_char_to_number(buffer[5]);
        Ok(Color { r, g, b })
    }
}

fn hex_char_to_number(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceAvailable {
    Yes,
    No,
    Unknown,
}

impl<'a> csvelo::ParseCsvField<'a> for ServiceAvailable {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        match buffer.trim_ascii() {
            b"1" => Ok(ServiceAvailable::Yes),
            b"0" => Ok(ServiceAvailable::No),
            _ => Ok(ServiceAvailable::Unknown),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl<'a> csvelo::ParseCsvField<'a> for Date {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        let buffer = buffer.trim_ascii();
        if buffer.len() != 8 {
            return Err(());
        }

        let year = (parse_digit(buffer[0]) as u16) * 1000
            + (parse_digit(buffer[1]) as u16) * 100
            + (parse_digit(buffer[2]) as u16) * 10
            + (parse_digit(buffer[3]) as u16);
        let month = parse_digit(buffer[4]) * 10 + parse_digit(buffer[5]);
        let day = parse_digit(buffer[6]) * 10 + parse_digit(buffer[7]);
        Ok(Date { year, month, day })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExceptionType {
    Added,
    Removed,
    Unknown,
}

impl<'a> csvelo::ParseCsvField<'a> for ExceptionType {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        match buffer.trim_ascii() {
            b"1" => Ok(ExceptionType::Added),
            b"2" => Ok(ExceptionType::Removed),
            _ => Ok(ExceptionType::Unknown),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YesOrNo {
    Yes,
    No,
    Unknown,
}

impl<'a> csvelo::ParseCsvField<'a> for YesOrNo {
    fn parse_csv_field(buffer: &'a [u8]) -> std::result::Result<Self, ()>
    where
        Self: 'a,
    {
        match buffer.trim_ascii() {
            b"" | b"0" => Ok(YesOrNo::No),
            b"1" => Ok(YesOrNo::Yes),
            _ => Ok(YesOrNo::Unknown),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OptionalF32(pub Option<f32>);

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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ServiceDayTime {
    seconds: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct OptionalServiceDayTime(pub Option<ServiceDayTime>);

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

fn parse_two_digit_int(buffer: &[u8]) -> u8 {
    return parse_digit(buffer[0]) * 10 + parse_digit(buffer[1]);
}

fn parse_digit(c: u8) -> u8 {
    c.wrapping_sub(b'0')
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
