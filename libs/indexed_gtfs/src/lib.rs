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

#[derive(Debug, Clone)]
pub struct GtfsFilter {
    pub stop_times: bool,
    pub stops: bool,
    pub trips: bool,
    pub routes: bool,
    pub calendar: bool,
    pub calendar_dates: bool,
    pub agencies: bool,
    pub feed_infos: bool,
    pub attributions: bool,
}

impl GtfsFilter {
    pub fn all() -> Self {
        Self {
            stop_times: true,
            stops: true,
            trips: true,
            routes: true,
            calendar: true,
            calendar_dates: true,
            agencies: true,
            feed_infos: true,
            attributions: true,
        }
    }

    pub fn none() -> Self {
        Self {
            stop_times: false,
            stops: false,
            trips: false,
            routes: false,
            calendar: false,
            calendar_dates: false,
            agencies: false,
            feed_infos: false,
            attributions: false,
        }
    }
}

impl Default for GtfsFilter {
    fn default() -> Self {
        Self::all()
    }
}

impl GtfsBuffers {
    /// Loads the GTFS either from a directory or a zip file.
    pub fn from_path(gtfs_path: &Path, filter: &GtfsFilter) -> Result<Self> {
        if gtfs_path.is_dir() {
            Ok(Self::from_dir(gtfs_path, filter))
        } else {
            Self::from_zip_file_path(gtfs_path, filter)
        }
    }

    /// Load available GTFS files from the given directory.
    pub fn from_dir(gtfs_dir: &Path, filter: &GtfsFilter) -> Self {
        macro_rules! load_from_dir {
            ($name:ident) => {
                if filter.$name {
                    std::fs::read(gtfs_dir.join(format!("{}.txt", stringify!($name)))).ok()
                } else {
                    None
                }
            };
        }

        Self {
            stop_times: load_from_dir!(stop_times),
            stops: load_from_dir!(stops),
            trips: load_from_dir!(trips),
            routes: load_from_dir!(routes),
            calendar: load_from_dir!(calendar),
            calendar_dates: load_from_dir!(calendar_dates),
            agencies: load_from_dir!(agencies),
            feed_infos: load_from_dir!(feed_infos),
            attributions: load_from_dir!(attributions),
        }
    }

    /// Load the available GTFS files from a zip file.
    pub fn from_zip_file_path(gtfs_zip_path: &Path, filter: &GtfsFilter) -> Result<Self> {
        let file = std::fs::File::open(gtfs_zip_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        Ok(Self::from_zip_file(&mut archive, filter))
    }

    /// Load the available GTFS files from a zip file using memory-mapped IO.
    /// That can be slightly more efficient than [`Self::from_zip_file_path`] but
    /// is unsafe when the underlying file is changed while it is read.
    pub unsafe fn from_zip_file_path_mmap(
        gtfs_zip_path: &Path,
        filter: &GtfsFilter,
    ) -> Result<Self> {
        let file = std::fs::File::open(gtfs_zip_path)?;
        let mmap = memmap2::Mmap::map(&file)?;
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(mmap))?;
        Ok(Self::from_zip_file(&mut archive, filter))
    }

    /// Load the available GTFS files from a slice that contains a zip file.
    pub fn from_zip_file_buffer(gtfs_zip_buffer: &[u8], filter: &GtfsFilter) -> Result<Self> {
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(gtfs_zip_buffer))?;
        Ok(Self::from_zip_file(&mut archive, filter))
    }

    pub fn from_zip_file<R: Read + Seek>(
        archive: &mut zip::ZipArchive<R>,
        filter: &GtfsFilter,
    ) -> Self {
        macro_rules! load_from_zip {
            ($name:ident) => {
                if filter.$name {
                    Self::read_archive_file(archive, &format!("{}.txt", stringify!($name))).ok()
                } else {
                    None
                }
            };
        }
        Self {
            stop_times: load_from_zip!(stop_times),
            stops: load_from_zip!(stops),
            trips: load_from_zip!(trips),
            routes: load_from_zip!(routes),
            calendar: load_from_zip!(calendar),
            calendar_dates: load_from_zip!(calendar_dates),
            agencies: load_from_zip!(agencies),
            feed_infos: load_from_zip!(feed_infos),
            attributions: load_from_zip!(attributions),
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
    pub unsafe fn from_dir(gtfs_dir: &Path, filter: &GtfsFilter) -> Self {
        macro_rules! load_from_dir_mmap {
            ($name:ident) => {
                if filter.$name {
                    Self::load(gtfs_dir, &format!("{}.txt", stringify!($name)))
                } else {
                    None
                }
            };
        }
        Self {
            stop_times: load_from_dir_mmap!(stop_times),
            stops: load_from_dir_mmap!(stops),
            trips: load_from_dir_mmap!(trips),
            routes: load_from_dir_mmap!(routes),
            calendar: load_from_dir_mmap!(calendar),
            calendar_dates: load_from_dir_mmap!(calendar_dates),
            agencies: load_from_dir_mmap!(agencies),
            feed_infos: load_from_dir_mmap!(feed_infos),
            attributions: load_from_dir_mmap!(attributions),
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
