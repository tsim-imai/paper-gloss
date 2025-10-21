use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t075_contract_delete_term_returns_204_for_existing() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Create term first
    let create_url = format!("{}/terms", api(&srv.base_url));
    let body = serde_json::json!({
        "lemma_en": "optimizer",
        "lemma_ja": "最適化手法"
    });
    let created = client.post(&create_url).json(&body).send().await.unwrap();
    assert_eq!(created.status().as_u16(), 201);
    let id = created.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().to_string();

    // Delete
    let url = format!("{}/terms/{}", api(&srv.base_url), id);
    let resp = client.delete(&url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 204, "expected 204 No Content per OpenAPI");
}
