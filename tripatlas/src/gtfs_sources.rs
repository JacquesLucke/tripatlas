use anyhow::{anyhow, Result};
use byte_unit::Byte;
use std::{
    collections::HashSet,
    ffi::OsStr,
    path::{Path, PathBuf},
};

/// Get a list of GTFS sources from the given input path.
/// The function detects automatically if the passed in path is itself a GTFS dataset or a
/// directory that contains potentially multiple GTFS datasets.
pub fn get_gtfs_sources(input_path: &Path, deduplicate_archives: bool) -> Vec<PathBuf> {
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

/// Sort GTFS sources by size so that the largest data-set is processed first. This improves
/// CPU utilization at then end of the analysis when there are multiple GTFS sources.
pub fn sort_gtfs_sources_by_size(mut gtfs_sources: Vec<PathBuf>) -> Vec<PathBuf> {
    gtfs_sources.sort_by_key(|p| get_estimated_dataset_size(p).unwrap_or(Byte::from_u64(0)));
    gtfs_sources.reverse();
    gtfs_sources
}

fn maybe_gtfs_dir(path: &std::path::Path) -> bool {
    path.is_dir() && path.join("stops.txt").exists()
}

fn maybe_gtfs_zip_file(path: &std::path::Path) -> bool {
    path.extension() == Some("zip".as_ref())
}

/// Estimates the size of the data set by adding up the size of the files that make it up.
pub fn get_estimated_dataset_size(path: &std::path::Path) -> Result<Byte> {
    Ok(Byte::from_u64(if path.is_dir() {
        fs_extra::dir::get_size(path).map_err(|e| anyhow!(e))?
    } else {
        std::fs::metadata(path)
            .map(|m| m.len())
            .map_err(|e| anyhow!(e))?
    }))
}
