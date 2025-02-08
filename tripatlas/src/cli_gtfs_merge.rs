use anyhow::Result;
use std::path::Path;

pub async fn gtfs_merge(input_path: &Path, output_path: &Path) -> Result<()> {
    println!(
        "Merging GTFS datasets from {:?} to {:?}",
        input_path, output_path
    );
    std::fs::create_dir_all(output_path)?;
    Ok(())
}
