mod structures;

use std::path::Path;

pub use structures::*;

impl<'a> Gtfs<'a> {
    pub fn from_buffers(buffers: GtfsBuffers<'a>) -> std::result::Result<Self, ()> {
        #[macro_export]
        macro_rules! do_parse {
            ($name:ident, $ty:ty) => {
                match buffers.$name {
                    Some(buffer) => match <$ty>::from_csv_buffer(buffer) {
                        Ok((data, len)) => File {
                            len,
                            data: Some(data),
                        },
                        Err(_) => File { len: 0, data: None },
                    },
                    None => File { len: 0, data: None },
                }
            };
        }

        Ok(Self {
            stop_times: do_parse!(stop_times, StopTimes),
            stops: do_parse!(stops, Stops),
            trips: do_parse!(trips, Trips),
            routes: do_parse!(routes, Routes),
            calendar: do_parse!(calendar, Calendar),
            calendar_dates: do_parse!(calendar_dates, CalendarDates),
            agencies: do_parse!(agencies, Agencies),
            feed_infos: do_parse!(feed_infos, FeedInfos),
            attributions: do_parse!(attributions, Attributions),
        })
    }
}

pub struct GtfsBuffers<'a> {
    pub stop_times: Option<&'a [u8]>,
    pub stops: Option<&'a [u8]>,
    pub trips: Option<&'a [u8]>,
    pub routes: Option<&'a [u8]>,
    pub calendar: Option<&'a [u8]>,
    pub calendar_dates: Option<&'a [u8]>,
    pub agencies: Option<&'a [u8]>,
    pub feed_infos: Option<&'a [u8]>,
    pub attributions: Option<&'a [u8]>,
}

pub struct GtfsBuffersRAM {
    pub stop_times: Option<Vec<u8>>,
    pub stops: Option<Vec<u8>>,
    pub trips: Option<Vec<u8>>,
    pub routes: Option<Vec<u8>>,
    pub calendar: Option<Vec<u8>>,
    pub calendar_dates: Option<Vec<u8>>,
    pub agencies: Option<Vec<u8>>,
    pub feed_infos: Option<Vec<u8>>,
    pub attributions: Option<Vec<u8>>,
}

impl GtfsBuffersRAM {
    pub fn from_dir(gtfs_dir: &Path) -> Self {
        Self {
            stop_times: std::fs::read(gtfs_dir.join("stop_times.txt")).ok(),
            stops: std::fs::read(gtfs_dir.join("stops.txt")).ok(),
            trips: std::fs::read(gtfs_dir.join("trips.txt")).ok(),
            routes: std::fs::read(gtfs_dir.join("routes.txt")).ok(),
            calendar: std::fs::read(gtfs_dir.join("calendar.txt")).ok(),
            calendar_dates: std::fs::read(gtfs_dir.join("calendar_dates.txt")).ok(),
            agencies: std::fs::read(gtfs_dir.join("agency.txt")).ok(),
            feed_infos: std::fs::read(gtfs_dir.join("feed_info.txt")).ok(),
            attributions: std::fs::read(gtfs_dir.join("attributions.txt")).ok(),
        }
    }

    pub fn to_slices<'a>(&'a self) -> GtfsBuffers<'a> {
        GtfsBuffers {
            stop_times: self.stop_times.as_ref().map(|s| &s[..]),
            stops: self.stops.as_ref().map(|s| &s[..]),
            trips: self.trips.as_ref().map(|s| &s[..]),
            routes: self.routes.as_ref().map(|s| &s[..]),
            calendar: self.calendar.as_ref().map(|s| &s[..]),
            calendar_dates: self.calendar_dates.as_ref().map(|s| &s[..]),
            agencies: self.agencies.as_ref().map(|s| &s[..]),
            feed_infos: self.feed_infos.as_ref().map(|s| &s[..]),
            attributions: self.attributions.as_ref().map(|s| &s[..]),
        }
    }
}

pub struct GtfsBuffersMmap {
    pub stop_times: Option<memmap2::Mmap>,
    pub stops: Option<memmap2::Mmap>,
    pub trips: Option<memmap2::Mmap>,
    pub routes: Option<memmap2::Mmap>,
    pub calendar: Option<memmap2::Mmap>,
    pub calendar_dates: Option<memmap2::Mmap>,
    pub agencies: Option<memmap2::Mmap>,
    pub feed_infos: Option<memmap2::Mmap>,
    pub attributions: Option<memmap2::Mmap>,
}

impl GtfsBuffersMmap {
    pub unsafe fn from_dir(gtfs_dir: &Path) -> Self {
        Self {
            stop_times: Self::load(gtfs_dir, "stop_times.txt"),
            stops: Self::load(gtfs_dir, "stops.txt"),
            trips: Self::load(gtfs_dir, "trips.txt"),
            routes: Self::load(gtfs_dir, "routes.txt"),
            calendar: Self::load(gtfs_dir, "calendar.txt"),
            calendar_dates: Self::load(gtfs_dir, "calendar_dates.txt"),
            agencies: Self::load(gtfs_dir, "agency.txt"),
            feed_infos: Self::load(gtfs_dir, "feed_info.txt"),
            attributions: Self::load(gtfs_dir, "attributions.txt"),
        }
    }

    unsafe fn load(gtfs_dir: &Path, file_name: &str) -> Option<memmap2::Mmap> {
        memmap2::Mmap::map(&std::fs::File::open(gtfs_dir.join(file_name)).unwrap()).ok()
    }
}

impl GtfsBuffersMmap {
    pub fn to_slices<'a>(&'a self) -> GtfsBuffers<'a> {
        GtfsBuffers {
            stop_times: self.stop_times.as_ref().map(|s| &s[..]),
            stops: self.stops.as_ref().map(|s| &s[..]),
            trips: self.trips.as_ref().map(|s| &s[..]),
            routes: self.routes.as_ref().map(|s| &s[..]),
            calendar: self.calendar.as_ref().map(|s| &s[..]),
            calendar_dates: self.calendar_dates.as_ref().map(|s| &s[..]),
            agencies: self.agencies.as_ref().map(|s| &s[..]),
            feed_infos: self.feed_infos.as_ref().map(|s| &s[..]),
            attributions: self.attributions.as_ref().map(|s| &s[..]),
        }
    }
}
