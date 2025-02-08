use num_format::ToFormattedString;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

#[derive(Debug, Default, Clone)]
struct Counts {
    stops: usize,
    parent_stops: usize,
    trips: usize,
    routes: usize,
    stop_times: usize,
    locations: Vec<(f32, f32)>,
}

pub async fn test_loader(test_dir: &Path) -> Result<()> {
    let mut gtfs_dirs: Vec<PathBuf> = test_dir
        .read_dir()?
        .filter(|e| match e {
            Ok(e) => match e.file_type() {
                Ok(t) => t.is_dir(),
                Err(_) => false,
            },
            Err(_) => false,
        })
        .map(|e| e.unwrap().path())
        .collect();
    let mut rng = rand::rng();
    gtfs_dirs.shuffle(&mut rng);

    let start_time = std::time::Instant::now();

    let individual_counts = gtfs_dirs
        .par_iter()
        .enumerate()
        .map(|(i, p)| {
            println!("{} Loading {:?}", i, p);
            let buffers = unsafe { indexed_gtfs::GtfsBuffersMmap::from_dir(&p) };
            let gtfs = indexed_gtfs::Gtfs::from_buffers(buffers.to_slices())
                .map_err(|e| anyhow!("{:?}", e))
                .unwrap();
            let locations = match &gtfs.stops.data {
                Some(data) => match (&data.stop_lon, &data.stop_lat) {
                    (Some(lngs), Some(lats)) => lngs
                        .iter()
                        .zip(lats)
                        .map(|(lng, lat)| (*lng, *lat))
                        .filter(|(lng, lat)| lng.0.is_some() && lat.0.is_some())
                        .map(|(lng, lat)| (lng.0.unwrap(), lat.0.unwrap()))
                        .collect(),
                    _ => vec![],
                },
                None => vec![],
            };
            let counts = Counts {
                stops: gtfs.stops.len,
                parent_stops: match gtfs.stops.data {
                    Some(data) => match data.parent_station {
                        Some(parent_station) => {
                            parent_station.iter().filter(|s| s.is_empty()).count()
                        }
                        None => 0,
                    },

                    None => 0,
                },
                trips: gtfs.trips.len,
                routes: gtfs.routes.len,
                stop_times: gtfs.stop_times.len,
                locations: locations,
            };
            counts
        })
        .collect::<Vec<_>>();
    let total_counts = individual_counts
        .iter()
        .cloned()
        .reduce(|acc, c| Counts {
            stops: acc.stops + c.stops,
            parent_stops: acc.parent_stops + c.parent_stops,
            trips: acc.trips + c.trips,
            routes: acc.routes + c.routes,
            stop_times: acc.stop_times + c.stop_times,
            locations: acc.locations.into_iter().chain(c.locations).collect(),
        })
        .unwrap();

    let obj_path = test_dir.join("found_locations.obj");
    let mut obj_file = std::fs::File::create(obj_path)?;
    obj_file.write_all(b"o Locations\n")?;
    for loc in &total_counts.locations {
        obj_file.write_all(format!("v {} {} 0\n", loc.0, loc.1).as_bytes())?;
    }

    println!(
        "Stops: {:#?}",
        total_counts
            .locations
            .len()
            .to_formatted_string(&num_format::Locale::en)
    );

    println!("Took {:?}", start_time.elapsed());

    Ok(())
}
