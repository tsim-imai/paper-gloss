use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t074_contract_patch_term_update_returns_200_for_existing() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Create term first
    let create_url = format!("{}/terms", api(&srv.base_url));
    let body = serde_json::json!({
        "lemma_en": "gradient descent",
        "lemma_ja": "勾配降下法"
    });
    let created = client.post(&create_url).json(&body).send().await.unwrap();
    assert_eq!(created.status().as_u16(), 201);
    let id = created.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().to_string();

    // Update
    let url = format!("{}/terms/{}", api(&srv.base_url), id);
    let body = serde_json::json!({ "note": "updated via test" });
    let resp = client.patch(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
}
