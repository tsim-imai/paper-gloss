use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t049_contract_get_term_detail_returns_200_for_existing() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // First create a term
    let create_url = format!("{}/terms", api(&srv.base_url));
    let body = serde_json::json!({
        "lemma_en": "neural network",
        "lemma_ja": "ニューラルネットワーク"
    });
    let created = client.post(&create_url).json(&body).send().await.unwrap();
    assert_eq!(created.status().as_u16(), 201);
    let json: serde_json::Value = created.json().await.unwrap();
    let term_id = json["id"].as_str().unwrap();

    // Then fetch detail (200)
    let url = format!("{}/terms/{}", api(&srv.base_url), term_id);
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
}
