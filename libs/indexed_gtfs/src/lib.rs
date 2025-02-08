mod structures;

use anyhow::Result;
use std::{
    io::{Read, Seek},
    path::Path,
};

pub use structures::*;

impl<'a> Gtfs<'a> {
    /// Parses the provided buffers into GTFS data.
    pub fn from_buffers(buffers: GtfsBufferSlices<'a>) -> Result<Self> {
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
            calendars: do_parse!(calendar, Calendar),
            calendar_dates: do_parse!(calendar_dates, CalendarDates),
            agencies: do_parse!(agencies, Agencies),
            feed_infos: do_parse!(feed_infos, FeedInfos),
            attributions: do_parse!(attributions, Attributions),
        })
    }
}

/// Contains references to buffers which generally wrap the .txt files in a GTFS archive.
/// This is usually created with [`GtfsBuffers::from_dir`] or [`GtfsBuffersMmap::from_dir`]
/// and their `.to_slices()` method.
#[derive(Debug)]
pub struct GtfsBufferSlices<'a> {
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

/// Owns a vector for each file in a GTFS archive.
pub struct GtfsBuffers {
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

impl GtfsBuffers {
    /// Load available GTFS files from the given directory.
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

    /// Load the available GTFS files from a zip file.
    pub fn from_zip_file_path(gtfs_zip_path: &Path) -> Result<Self> {
        let file = std::fs::File::open(gtfs_zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        Ok(Self::from_zip_file(&mut archive))
    }

    /// Load the available GTFS files from a zip file using memory-mapped IO.
    /// That can be slightly more efficient than [`Self::from_zip_file_path`] but
    /// is unsafe when the underlying file is changed while it is read.
    pub unsafe fn from_zip_file_path_mmap(gtfs_zip_path: &Path) -> Result<Self> {
        let file = std::fs::File::open(gtfs_zip_path)?;
        let mmap = memmap2::Mmap::map(&file)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(mmap))?;
        Ok(Self::from_zip_file(&mut archive))
    }

    /// Load the available GTFS files from a slice that contains a zip file.
    pub fn from_zip_file_buffer(gtfs_zip_buffer: &[u8]) -> Result<Self> {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(gtfs_zip_buffer))?;
        Ok(Self::from_zip_file(&mut archive))
    }

    pub fn from_zip_file<R: Read + Seek>(archive: &mut zip::ZipArchive<R>) -> Self {
        Self {
            stop_times: Self::read_archive_file(archive, "stop_times.txt").ok(),
            stops: Self::read_archive_file(archive, "stops.txt").ok(),
            trips: Self::read_archive_file(archive, "trips.txt").ok(),
            routes: Self::read_archive_file(archive, "routes.txt").ok(),
            calendar: Self::read_archive_file(archive, "calendar.txt").ok(),
            calendar_dates: Self::read_archive_file(archive, "calendar_dates.txt").ok(),
            agencies: Self::read_archive_file(archive, "agency.txt").ok(),
            feed_infos: Self::read_archive_file(archive, "feed_info.txt").ok(),
            attributions: Self::read_archive_file(archive, "attributions.txt").ok(),
        }
    }

    fn read_archive_file<R: Read + Seek>(
        archive: &mut zip::ZipArchive<R>,
        file_name: &str,
    ) -> Result<Vec<u8>> {
        let mut file = archive.by_name(file_name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Get the slices owned by this instance to use with [`Gtfs::from_buffers`].
    pub fn to_slices<'a>(&'a self) -> GtfsBufferSlices<'a> {
        GtfsBufferSlices {
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

/// Similar to [`GtfsBuffers`] but does not make copies of the buffers.
/// This can be much more efficient with large datasets but is unsafe when
/// the underlying file is changed while it is read.
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
    /// Load available GTFS files from the given directory.
    /// This can be much more efficient with large datasets but is unsafe when
    /// the underlying file is changed while it is read.
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
        match std::fs::File::open(gtfs_dir.join(file_name)) {
            Ok(f) => memmap2::Mmap::map(&f).ok(),
            Err(_) => None,
        }
    }
}

impl GtfsBuffersMmap {
    /// Get the slices owned by this instance to use with [`Gtfs::from_buffers`].
    pub fn to_slices<'a>(&'a self) -> GtfsBufferSlices<'a> {
        GtfsBufferSlices {
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
