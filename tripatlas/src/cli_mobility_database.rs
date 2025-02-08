use anyhow::Result;
use colored::Colorize;
use futures::executor::block_on_stream;
use genawaiter::{rc::gen, yield_};
use std::path::Path;

use crate::util;

const MOBILITY_DATABASE_URL: &str = "https://api.mobilitydatabase.org/v1/";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct GtfsFeedInfo {
    id: String,
    data_type: String,
    created_at: String,
    official: Option<bool>,
    provider: String,
    source_info: GtfsFeedSourceInfo,
    latest_dataset: Option<GtfsFeedLatestDataset>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct GtfsFeedSourceInfo {
    authentication_type: Option<i32>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct GtfsFeedLatestDataset {
    id: String,
    hosted_url: String,
    bounding_box: Option<GtfsFeedBoundingBox>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct GtfsFeedBoundingBox {
    minimum_latitude: f64,
    maximum_latitude: f64,
    minimum_longitude: f64,
    maximum_longitude: f64,
}

pub async fn download_mobility_database_gtfs(
    token: &str,
    out_dir: &Path,
    limit: usize,
) -> Result<()> {
    if limit == 0 {
        return Ok(());
    }
    std::fs::create_dir_all(out_dir)?;

    let gtfs_feeds = gen!({
        let chunk_size = 100;
        let mut offset = 0;
        loop {
            let feeds = match load_gtfs_feeds_chunk(&token, offset, chunk_size).await {
                Ok(feeds) => feeds,
                Err(e) => {
                    println!("{}", format!("Error loading feeds:\n{:?}", e).red());
                    break;
                }
            };
            if feeds.is_empty() {
                break;
            }
            offset += feeds.len();
            for feed in feeds {
                yield_!(feed);
            }
        }
    });

    let mut downloaded_count = 0;
    for (i, feed) in block_on_stream(gtfs_feeds).enumerate() {
        println!("{}: {:?}", i, feed.id);
        println!("  Provider: {:?}", feed.provider);
        match &feed.latest_dataset {
            Some(latest_dataset) => {
                let dataset_id = &latest_dataset.id;
                let gtfs_url = &latest_dataset.hosted_url;
                let filename = format!("{}.zip", dataset_id);
                let output_path = out_dir.join(filename);
                if output_path.exists() {
                    println!("  {}", "Already downloaded.".yellow());
                    continue;
                }
                let Ok(res) = reqwest::get(gtfs_url).await else {
                    println!("  {}", format!("Error accessing URL: {:?}", gtfs_url).red());
                    continue;
                };
                if !res.status().is_success() {
                    println!("  {}", format!("Got error code: {:?}", res.status()).red());
                    continue;
                }
                let Some(bytes) = res.bytes().await.ok() else {
                    println!(
                        "  {}",
                        format!("Error downloading content: {:?}", gtfs_url).red()
                    );
                    continue;
                };
                if (std::fs::write(&output_path, &bytes)).is_err() {
                    println!("  {}", "Error writing file.".red());
                    continue;
                }
                println!("  Saved at {:?}", output_path);
                println!(
                    "  Size: {}",
                    util::bytes_to_human_string(bytes.len() as u64).green()
                );
                downloaded_count += 1;
                if downloaded_count >= limit {
                    println!("Limit reached.");
                    return Ok(());
                }
            }
            None => {
                println!("  {}", "No latest dataset found.".yellow());
            }
        }
    }
    println!("All the latest datasets are downloaded.");
    Ok(())
}

async fn load_gtfs_feeds_chunk(
    token: &str,
    offset: usize,
    chunk_size: usize,
) -> Result<Vec<GtfsFeedInfo>> {
    let client = reqwest::Client::new();
    let feeds_url = format!(
        "{}gtfs_feeds?limit={}&offset={}",
        MOBILITY_DATABASE_URL, chunk_size, offset
    );
    let res = client.get(feeds_url).bearer_auth(token).send().await?;
    res.json::<Vec<GtfsFeedInfo>>().await.map_err(|e| e.into())
}
