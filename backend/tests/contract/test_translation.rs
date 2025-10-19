use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t021_contract_get_translation_returns_200_with_chunks() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let paper_id = uuid::Uuid::new_v4();
    let url = format!("{}/papers/{}/translation", api(&srv.base_url), paper_id);
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
    let json: serde_json::Value = resp.json().await.unwrap();
    assert!(json.get("chunks").is_some(), "missing chunks array");
}
