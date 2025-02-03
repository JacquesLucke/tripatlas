use std::net::TcpListener;

struct TestContext {
    handle: tokio::task::JoinHandle<()>,
    url: String,
    client: reqwest::Client,
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn setup() -> TestContext {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://127.0.0.1:{}", port);

    let server = tokio::spawn(async move {
        crate::start_server::start_server(listener)
            .await
            .expect("Failed to start server");
    });

    // Wait for server to start.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    TestContext {
        handle: server,
        url,
        client: reqwest::Client::new(),
    }
}

impl TestContext {
    async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(format!("{}{}", self.url, path))
            .send()
            .await
            .unwrap()
    }
}

#[tokio::test]
async fn index_html_is_served() {
    let ctx = setup().await;
    let response = ctx.get("/").await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let text = response.text().await.unwrap();
    assert!(text.contains("<!DOCTYPE html>"));
}

#[tokio::test]
async fn api_root_is_served() {
    let ctx = setup().await;
    let response = ctx.get("/api").await;
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let text = response.text().await.unwrap();
    assert_eq!(text, "The api is working.");
}
