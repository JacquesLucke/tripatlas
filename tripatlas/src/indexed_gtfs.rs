use indexed_gtfs::*;
use std::{path::Path, time::Instant};

use num_format::ToFormattedString;

pub fn parse_performance_test() {
    let gtfs_dir = Path::new("/home/jacques/Documents/gtfs_germany");

    {
        let stop_times_timer = Instant::now();
        let stop_times_path = gtfs_dir.join("stop_times.txt");
        let stop_times_file = std::fs::File::open(stop_times_path).unwrap();
        let stop_times_mmap = unsafe { memmap2::Mmap::map(&stop_times_file) }.unwrap();
        let (_stop_times, records_num) = StopTimes::from_csv_buffer(&stop_times_mmap[..]).unwrap();
        println!(
            "Stop Times: {:?}, found: {}",
            stop_times_timer.elapsed(),
            records_num.to_formatted_string(&num_format::Locale::en)
        );
    }
    {
        let stops_timer = Instant::now();
        let stops_path = gtfs_dir.join("stops.txt");
        let stops_file = std::fs::File::open(stops_path).unwrap();
        let stops_mmap = unsafe { memmap2::Mmap::map(&stops_file) }.unwrap();
        let (_stops, records_num) = Stops::from_csv_buffer(&stops_mmap[..]).unwrap();
        println!(
            "Stop Times: {:?}, found: {}",
            stops_timer.elapsed(),
            records_num.to_formatted_string(&num_format::Locale::en)
        );
    }
    {
        let trips_timer = Instant::now();
        let trips_path = gtfs_dir.join("trips.txt");
        let trips_file = std::fs::File::open(trips_path).unwrap();
        let trips_mmap = unsafe { memmap2::Mmap::map(&trips_file) }.unwrap();
        let (_trips, records_num) = Trips::from_csv_buffer(&trips_mmap[..]).unwrap();
        println!(
            "Trips: {:?}, found: {}",
            trips_timer.elapsed(),
            records_num.to_formatted_string(&num_format::Locale::en)
        );
    }

    {
        let routes_timer = Instant::now();
        let routes_path = gtfs_dir.join("routes.txt");
        let routes_file = std::fs::File::open(routes_path).unwrap();
        let routes_mmap = unsafe { memmap2::Mmap::map(&routes_file) }.unwrap();
        let (_routes, records_num) = Routes::from_csv_buffer(&routes_mmap[..]).unwrap();
        println!(
            "Routes: {:?}, found: {}",
            routes_timer.elapsed(),
            records_num.to_formatted_string(&num_format::Locale::en)
        );
    }

    {
        let calendar_timer = Instant::now();
        let calendar_path = gtfs_dir.join("calendar.txt");
        let calendar_file = std::fs::File::open(calendar_path).unwrap();
        let calendar_mmap = unsafe { memmap2::Mmap::map(&calendar_file) }.unwrap();
        let (_calendar, records_num) = Calendar::from_csv_buffer(&calendar_mmap[..]).unwrap();
        println!(
            "Calendar: {:?}, found: {}",
            calendar_timer.elapsed(),
            records_num.to_formatted_string(&num_format::Locale::en)
        );
    }

    {
        let calendar_dates_timer = Instant::now();
        let calendar_dates_path = gtfs_dir.join("calendar_dates.txt");
        let calendar_dates_file = std::fs::File::open(calendar_dates_path).unwrap();
        let calendar_dates_mmap = unsafe { memmap2::Mmap::map(&calendar_dates_file) }.unwrap();
        let (_calendar_dates, records_num) =
            CalendarDates::from_csv_buffer(&calendar_dates_mmap[..]).unwrap();
        println!(
            "Calendar Dates: {:?}, found: {}",
            calendar_dates_timer.elapsed(),
            records_num.to_formatted_string(&num_format::Locale::en)
        );
    }

    {
        let agencies_timer = Instant::now();
        let agencies_path = gtfs_dir.join("agency.txt");
        let agencies_file = std::fs::File::open(agencies_path).unwrap();
        let agencies_mmap = unsafe { memmap2::Mmap::map(&agencies_file) }.unwrap();
        let (_agencies, records_num) = Agencies::from_csv_buffer(&agencies_mmap[..]).unwrap();
        println!(
            "Agencies: {:?}, found: {}",
            agencies_timer.elapsed(),
            records_num.to_formatted_string(&num_format::Locale::en)
        );
    }

    {
        let feed_infos_timer = Instant::now();
        let feed_infos_path = gtfs_dir.join("feed_info.txt");
        let feed_infos_file = std::fs::File::open(feed_infos_path).unwrap();
        let feed_infos_mmap = unsafe { memmap2::Mmap::map(&feed_infos_file) }.unwrap();
        let (_feed_infos, records_num) = FeedInfos::from_csv_buffer(&feed_infos_mmap[..]).unwrap();
        println!(
            "Feed Infos: {:?}, found: {}",
            feed_infos_timer.elapsed(),
            records_num.to_formatted_string(&num_format::Locale::en)
        );
    }

    {
        let attributions_timer = Instant::now();
        let attributions_path = gtfs_dir.join("attributions.txt");
        let attributions_file = std::fs::File::open(attributions_path).unwrap();
        let attributions_mmap = unsafe { memmap2::Mmap::map(&attributions_file) }.unwrap();
        let (_attributions, records_num) =
            Attributions::from_csv_buffer(&attributions_mmap[..]).unwrap();
        println!(
            "Attributions: {:?}, found: {}",
            attributions_timer.elapsed(),
            records_num.to_formatted_string(&num_format::Locale::en)
        );
    }
}
