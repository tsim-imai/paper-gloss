use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t049_contract_get_term_detail_returns_200() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let term_id = uuid::Uuid::new_v4();
    let url = format!("{}/terms/{}", api(&srv.base_url), term_id);
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
}
