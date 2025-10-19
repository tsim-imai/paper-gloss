use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t050_contract_list_occurrences_returns_200() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let paper_id = uuid::Uuid::new_v4();
    let url = format!("{}/occurrences?paper_id={}", api(&srv.base_url), paper_id);
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
    let _arr: serde_json::Value = resp.json().await.unwrap();
}
