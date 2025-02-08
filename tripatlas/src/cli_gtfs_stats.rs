use std::{collections::HashSet, ffi::OsStr, path::PathBuf};

use anyhow::Result;
use colored::Colorize;
use num_format::ToFormattedString;
use rayon::prelude::*;

use crate::util;

#[derive(Debug, Default)]
struct GtfsStats {
    stop_times_num: usize,
    stops_num: usize,
    trips_num: usize,
    routes_num: usize,
    calendars_num: usize,
    calendar_dates_num: usize,
    agencies_num: usize,
    feed_infos_num: usize,
    attributions_num: usize,
}

pub async fn gtfs_stats(input_path: &std::path::Path, deduplicate_archives: bool) -> Result<()> {
    let mut gtfs_sources = get_gtfs_sources(input_path, deduplicate_archives);
    if gtfs_sources.is_empty() {
        println!("No GTFS sources found.");
        return Ok(());
    }

    // Sort by size so that the largest data-set is loaded first. This improves CPU utilization
    // at then end of the analysis when there are multiple GTFS sources.
    gtfs_sources.sort_by_key(|p| get_estimated_dataset_size(p).unwrap_or(0));
    gtfs_sources.reverse();

    let counter = std::sync::atomic::AtomicUsize::new(0);

    let all_gtfs_stats: Vec<Result<GtfsStats>> = (0..gtfs_sources.len())
        .into_par_iter()
        .map(|_| {
            // Manually retrieve the work item, so that we can take the sorting done previously into account.
            let current = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let p = &gtfs_sources[current];
            println!(
                "{: >3}/{} {: >10} {}",
                current,
                gtfs_sources.len(),
                util::bytes_to_human_string(get_estimated_dataset_size(&p).unwrap_or(0)).green(),
                p.strip_prefix(input_path)
                    .unwrap_or(p)
                    .to_str()
                    .unwrap_or("")
            );
            if p.is_dir() {
                let buffers = unsafe { indexed_gtfs::GtfsBuffersMmap::from_dir(p) };
                let gtfs = indexed_gtfs::Gtfs::from_buffers(buffers.to_slices())?;
                Ok(analyse_gtfs(&gtfs))
            } else {
                let buffers = unsafe { indexed_gtfs::GtfsBuffers::from_zip_file_path_mmap(p) }?;
                let gtfs = indexed_gtfs::Gtfs::from_buffers(buffers.to_slices())?;
                Ok(analyse_gtfs(&gtfs))
            }
        })
        .collect();

    let merged_stats = all_gtfs_stats.into_iter().filter_map(|r| r.ok()).fold(
        GtfsStats {
            ..Default::default()
        },
        |acc, stats| GtfsStats {
            stop_times_num: acc.stop_times_num + stats.stop_times_num,
            stops_num: acc.stops_num + stats.stops_num,
            trips_num: acc.trips_num + stats.trips_num,
            routes_num: acc.routes_num + stats.routes_num,
            calendars_num: acc.calendars_num + stats.calendars_num,
            calendar_dates_num: acc.calendar_dates_num + stats.calendar_dates_num,
            agencies_num: acc.agencies_num + stats.agencies_num,
            feed_infos_num: acc.feed_infos_num + stats.feed_infos_num,
            attributions_num: acc.attributions_num + stats.attributions_num,
        },
    );

    println!("Total GTFS stats:");
    let locale = num_format::Locale::en;
    println!(
        "  Stop times: {}",
        merged_stats.stop_times_num.to_formatted_string(&locale)
    );
    println!(
        "  Stops: {}",
        merged_stats.stops_num.to_formatted_string(&locale)
    );
    println!(
        "  Trips: {}",
        merged_stats.trips_num.to_formatted_string(&locale)
    );
    println!(
        "  Routes: {}",
        merged_stats.routes_num.to_formatted_string(&locale)
    );
    println!(
        "  Calendars: {}",
        merged_stats.calendars_num.to_formatted_string(&locale)
    );
    println!(
        "  Calendar dates: {}",
        merged_stats.calendar_dates_num.to_formatted_string(&locale)
    );
    println!(
        "  Agencies: {}",
        merged_stats.agencies_num.to_formatted_string(&locale)
    );
    println!(
        "  Feed infos: {}",
        merged_stats.feed_infos_num.to_formatted_string(&locale)
    );
    println!(
        "  Attributions: {}",
        merged_stats.attributions_num.to_formatted_string(&locale)
    );

    Ok(())
}

fn get_gtfs_sources(input_path: &std::path::Path, deduplicate_archives: bool) -> Vec<PathBuf> {
    let mut gtfs_folders = vec![];
    let mut gtfs_zip_files = vec![];
    if input_path.is_dir() {
        if maybe_gtfs_dir(input_path) {
            gtfs_folders.push(input_path.to_path_buf());
        } else {
            let walker = walkdir::WalkDir::new(input_path).follow_links(true);
            for entry in walker {
                let Ok(entry) = entry else {
                    continue;
                };
                let path = entry.path();
                let file_type = entry.file_type();
                if file_type.is_dir() {
                    if maybe_gtfs_dir(&path) {
                        gtfs_folders.push(path.to_owned());
                        continue;
                    }
                } else if file_type.is_file() {
                    if maybe_gtfs_zip_file(path) {
                        gtfs_zip_files.push(path.to_owned());
                        continue;
                    }
                }
            }
        }
    } else if maybe_gtfs_zip_file(input_path) {
        gtfs_zip_files.push(input_path.to_owned());
    }

    if deduplicate_archives {
        let gtfs_folder_names: HashSet<&OsStr> =
            gtfs_folders.iter().filter_map(|p| p.file_name()).collect();
        gtfs_zip_files = gtfs_zip_files
            .into_iter()
            .filter(|p| match p.with_extension("").file_name() {
                Some(name) => {
                    if gtfs_folder_names.contains(name) {
                        println!("Ignored potential duplicate: {:?}", p);
                        false
                    } else {
                        true
                    }
                }
                None => false,
            })
            .collect();
    }

    gtfs_folders
        .into_iter()
        .chain(gtfs_zip_files.into_iter())
        .map(|p| p.to_owned())
        .collect()
}

fn maybe_gtfs_dir(path: &std::path::Path) -> bool {
    path.is_dir() && path.join("stops.txt").exists()
}

fn maybe_gtfs_zip_file(path: &std::path::Path) -> bool {
    path.extension() == Some("zip".as_ref())
}

fn get_estimated_dataset_size(path: &std::path::Path) -> Result<u64> {
    if path.is_dir() {
        fs_extra::dir::get_size(path).map_err(|e| e.into())
    } else {
        std::fs::metadata(path)
            .map(|m| m.len())
            .map_err(|e| e.into())
    }
}

fn analyse_gtfs(gtfs: &indexed_gtfs::Gtfs) -> GtfsStats {
    GtfsStats {
        stop_times_num: gtfs.stop_times.len,
        stops_num: gtfs.stops.len,
        trips_num: gtfs.trips.len,
        routes_num: gtfs.routes.len,
        calendars_num: gtfs.calendars.len,
        calendar_dates_num: gtfs.calendar_dates.len,
        agencies_num: gtfs.agencies.len,
        feed_infos_num: gtfs.feed_infos.len,
        attributions_num: gtfs.attributions.len,
    }
}
