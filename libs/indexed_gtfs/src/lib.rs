mod structures;

pub use structures::*;

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
