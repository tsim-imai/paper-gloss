use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t074_contract_patch_term_update_returns_200() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let term_id = uuid::Uuid::new_v4();
    let url = format!("{}/terms/{}", api(&srv.base_url), term_id);
    let body = serde_json::json!({
        "note": "updated via test"
    });
    let resp = reqwest::Client::new().patch(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
}
