use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_contract_paper_import_requires_multipart_and_returns_201() {
    // Start server
    let srv = TestServer::spawn().expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));

    // Per OpenAPI, request is multipart/form-data with either file or (url,title)
    // Here we exercise URL import path; expect 201 on success per contract.
    let form = reqwest::multipart::Form::new()
        .text("url", "https://arxiv.org/abs/2212.14578")
        .text("title", "Sample Paper");

    let resp = client.post(&url).multipart(form).send().await.unwrap();

    // Contract expectation (will FAIL until implemented): 201 Created
    assert_eq!(resp.status().as_u16(), 201, "expected 201 Created per OpenAPI");

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.get("paper_id").is_some(), "missing paper_id in response");
}
