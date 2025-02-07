use std::{path::Path, time::Instant};

pub fn parse_performance_test() {
    let gtfs_dir = Path::new("/home/jacques/Documents/gtfs_germany");

    let start_time = Instant::now();

    // let buffers_ram = indexed_gtfs::GtfsBuffersRAM::from_dir(&gtfs_dir);
    let buffers_mmap = unsafe { indexed_gtfs::GtfsBuffersMmap::from_dir(&gtfs_dir) };

    let gtfs = indexed_gtfs::Gtfs::from_buffers(buffers_mmap.to_slices());

    println!("Time elapsed: {:?}", start_time.elapsed());

    println!("{:#?}", gtfs);
}
