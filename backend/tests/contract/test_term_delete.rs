use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t075_contract_delete_term_returns_204() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let term_id = uuid::Uuid::new_v4();
    let url = format!("{}/terms/{}", api(&srv.base_url), term_id);
    let resp = reqwest::Client::new().delete(&url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 204, "expected 204 No Content per OpenAPI");
}
