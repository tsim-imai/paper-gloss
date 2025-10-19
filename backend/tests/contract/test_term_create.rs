use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t073_contract_post_terms_create_returns_201() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let url = format!("{}/terms", api(&srv.base_url));
    let body = serde_json::json!({
        "lemma_en": "neural network",
        "lemma_ja": "ニューラルネットワーク"
    });
    let resp = reqwest::Client::new().post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201, "expected 201 Created per OpenAPI");
}
