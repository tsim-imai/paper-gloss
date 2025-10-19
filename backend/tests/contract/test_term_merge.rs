use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t076_contract_post_terms_merge_returns_200() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let url = format!("{}/terms/merge", api(&srv.base_url));
    let body = serde_json::json!({
        "source_id": uuid::Uuid::new_v4().to_string(),
        "target_id": uuid::Uuid::new_v4().to_string(),
        "confirmed": true
    });
    let resp = reqwest::Client::new().post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
}
