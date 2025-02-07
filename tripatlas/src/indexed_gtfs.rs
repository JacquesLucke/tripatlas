use std::{path::Path, time::Instant};

pub fn parse_performance_test() {
    let gtfs_dir = Path::new("/home/jacques/Documents/gtfs_germany");

    let stop_times_path = gtfs_dir.join("stop_times.txt");
    let stops_path = gtfs_dir.join("stops.txt");
    let trips_path = gtfs_dir.join("trips.txt");
    let routes_path = gtfs_dir.join("routes.txt");
    let calendar_path = gtfs_dir.join("calendar.txt");
    let calendar_dates_path = gtfs_dir.join("calendar_dates.txt");
    let agencies_path = gtfs_dir.join("agency.txt");
    let feed_infos_path = gtfs_dir.join("feed_info.txt");
    let attributions_path = gtfs_dir.join("attributions.txt");

    let start_time = Instant::now();

    let stop_times_mmap =
        unsafe { memmap2::Mmap::map(&std::fs::File::open(stop_times_path).unwrap()) }.unwrap();
    let stops_mmap =
        unsafe { memmap2::Mmap::map(&std::fs::File::open(stops_path).unwrap()) }.unwrap();
    let trips_mmap =
        unsafe { memmap2::Mmap::map(&std::fs::File::open(trips_path).unwrap()) }.unwrap();
    let routes_mmap =
        unsafe { memmap2::Mmap::map(&std::fs::File::open(routes_path).unwrap()) }.unwrap();
    let calendar_mmap =
        unsafe { memmap2::Mmap::map(&std::fs::File::open(calendar_path).unwrap()) }.unwrap();
    let calendar_dates_mmap =
        unsafe { memmap2::Mmap::map(&std::fs::File::open(calendar_dates_path).unwrap()) }.unwrap();
    let agencies_mmap =
        unsafe { memmap2::Mmap::map(&std::fs::File::open(agencies_path).unwrap()) }.unwrap();
    let feed_infos_mmap =
        unsafe { memmap2::Mmap::map(&std::fs::File::open(feed_infos_path).unwrap()) }.unwrap();
    let attributions_mmap =
        unsafe { memmap2::Mmap::map(&std::fs::File::open(attributions_path).unwrap()) }.unwrap();

    let gtfs = indexed_gtfs::Gtfs::from_buffers(indexed_gtfs::GtfsBuffers {
        stop_times: Some(&stop_times_mmap[..]),
        stops: Some(&stops_mmap[..]),
        trips: Some(&trips_mmap[..]),
        routes: Some(&routes_mmap[..]),
        calendar: Some(&calendar_mmap[..]),
        calendar_dates: Some(&calendar_dates_mmap[..]),
        agencies: Some(&agencies_mmap[..]),
        feed_infos: Some(&feed_infos_mmap[..]),
        attributions: Some(&attributions_mmap[..]),
    });

    println!("Time elapsed: {:?}", start_time.elapsed());

    println!("{:#?}", gtfs);
}
