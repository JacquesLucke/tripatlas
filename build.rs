use anyhow::Result;
use duct::cmd;
use std::{path::Path, time::UNIX_EPOCH};

fn main() {
    rebuild_frontend_if_necessary();
}

fn rebuild_frontend_if_necessary() {
    let project_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let frontend_dir = project_dir.join("frontend");

    let frontend_change_time = find_latest_modification_time(&frontend_dir, &|e| {
        let path = e.path();
        !(path.ends_with("dist") || path.ends_with("node_modules"))
    });
    let build_change_time = find_latest_modification_time(&frontend_dir.join("dist"), &|_e| true);

    if let Ok(frontend_change_time) = frontend_change_time {
        if let Ok(build_change_time) = build_change_time {
            if build_change_time > frontend_change_time {
                // No need to rebuild the frontend.
                return;
            }
        }
    }

    cmd!("npm", "install").dir(&frontend_dir).run().unwrap();
    cmd!("npm", "run", "build")
        .dir(&frontend_dir)
        .run()
        .unwrap();
}

fn find_latest_modification_time(
    dir: &Path,
    filter: &dyn Fn(&walkdir::DirEntry) -> bool,
) -> Result<u64> {
    let mut latest = 0;
    for entry in walkdir::WalkDir::new(dir).into_iter().filter_entry(filter) {
        let entry = &entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let metadata = entry.metadata()?;
        let modified = metadata.modified()?;
        let modified_seconds = modified.duration_since(UNIX_EPOCH)?.as_secs();
        latest = latest.max(modified_seconds);
    }
    Ok(latest)
}
