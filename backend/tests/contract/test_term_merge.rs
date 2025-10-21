use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t076_contract_post_terms_merge_returns_200_for_existing_terms() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Create source and target terms
    let create_url = format!("{}/terms", api(&srv.base_url));
    let src = client.post(&create_url).json(&serde_json::json!({
        "lemma_en": "cnn",
        "lemma_ja": "畳み込みニューラルネットワーク"
    })).send().await.unwrap().json::<serde_json::Value>().await.unwrap();
    let tgt = client.post(&create_url).json(&serde_json::json!({
        "lemma_en": "convolutional neural network",
        "lemma_ja": "畳み込みニューラルネットワーク"
    })).send().await.unwrap().json::<serde_json::Value>().await.unwrap();

    let body = serde_json::json!({
        "source_id": src["id"].as_str().unwrap(),
        "target_id": tgt["id"].as_str().unwrap(),
        "confirmed": true
    });
    let url = format!("{}/terms/merge", api(&srv.base_url));
    let resp = client.post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
}
