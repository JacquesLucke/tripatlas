use std::path::Path;

use anyhow::{anyhow, Result};

pub async fn test_loader(test_dir: &Path) -> Result<()> {
    let gtfs_dirs = test_dir.read_dir()?;
    for gtfs_dir in gtfs_dirs {
        let gtfs_dir = gtfs_dir?;
        if !gtfs_dir.file_type()?.is_dir() {
            continue;
        }
        println!("Loading {}", gtfs_dir.path().display());
        let gtfs_dir = gtfs_dir.path();
        let buffers_ram = indexed_gtfs::GtfsBuffersRAM::from_dir(&gtfs_dir);
        let gtfs = indexed_gtfs::Gtfs::from_buffers(buffers_ram.to_slices())
            .map_err(|e| anyhow!("{:?}", e))?;
        println!("{:#?}", gtfs);
    }
    Ok(())
}
