use crate::common::*;

/// US2: After processing a paper, terms should be extractable and highlightable
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t051_integration_term_extraction_to_highlighting() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let client = reqwest::Client::new();

    // Import & process a paper (will fail until implemented)
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    let form = reqwest::multipart::Form::new()
        .text("url", "https://arxiv.org/abs/2212.14578")
        .text("title", "Sample");
    let resp = client.post(&import_url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201);
    let paper_id = resp.json::<serde_json::Value>().await.unwrap()["paper_id"].as_str().unwrap().to_string();

    let process_url = format!("{}/papers/{}/process", api(&srv.base_url), paper_id);
    let _ = client.post(&process_url).send().await.unwrap();

    // Terms should be listable and occurrences available
    let terms_url = format!("{}/terms?q=network", api(&srv.base_url));
    let resp = reqwest::get(&terms_url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}
