use anyhow::Result;
use rayon::result;
use serde::Deserialize;
use serde_json::json;

const MOBILITY_DATABASE_URL: &str = "https://api.mobilitydatabase.org/v1/";

pub async fn test_loading_data(token: &str) -> Result<()> {
    let feeds = load_all_feeds(token).await?;
    std::fs::write(
        "/home/jacques/Documents/feeds_test_response.json",
        json!(feeds).to_string(),
    )?;
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
