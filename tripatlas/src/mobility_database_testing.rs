use std::path::Path;

use anyhow::Result;

const MOBILITY_DATABASE_URL: &str = "https://api.mobilitydatabase.org/v1/";

pub async fn test_loading_data(token: &str) -> Result<()> {
    let cache_file = "/home/jacques/Documents/feeds_test_response.json";
    let cache_dir = Path::new("/home/jacques/Documents/gtfs_all_datasets");
    let feeds: Vec<GtfsFeedInfo> = serde_json::from_str(&std::fs::read_to_string(cache_file)?)?;
    for (i, feed) in feeds.iter().enumerate() {
        if let Some(latest_dataset) = &feed.latest_dataset {
            let gtfs_url = &latest_dataset.hosted_url;
            let filename = gtfs_url
                .replace(":", "_")
                .replace("/", "_")
                .replace("-", "_");
            let output_path = cache_dir.join(filename);
            println!("{} {}", i, latest_dataset.hosted_url);
            if output_path.exists() {
                println!("Already downloaded.");
                continue;
            }
            if let Ok(res) = reqwest::get(gtfs_url).await {
                if res.status().is_success() {
                    if let Some(res) = res.bytes().await.ok() {
                        let _ = std::fs::write(output_path, res);
                        continue;
                    }
                }
            }
            println!("Failed to download.");
        }
    }

    // let feeds = load_all_feeds(token).await?;
    // std::fs::write(
    //     "/home/jacques/Documents/feeds_test_response.json",
    //     json!(feeds).to_string(),
    // )?;
    Ok(())
}

async fn load_all_feeds(token: &str) -> Result<Vec<GtfsFeedInfo>> {
    let chunk_size = 500;
    let mut result = vec![];
    loop {
        let feeds = load_feeds_chunk(token, result.len(), chunk_size).await?;
        if feeds.is_empty() {
            break;
        }
        result.extend(feeds);
        println!("Loaded {} feeds so far.", result.len());
    }
    Ok(result)
}

async fn load_feeds_chunk(
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
