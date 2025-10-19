use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t023_contract_post_chunk_retry_returns_202() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let chunk_id = uuid::Uuid::new_v4();
    let url = format!("{}/chunks/{}/retry", api(&srv.base_url), chunk_id);
    let resp = reqwest::Client::new().post(&url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 202, "expected 202 Accepted per OpenAPI");
}
