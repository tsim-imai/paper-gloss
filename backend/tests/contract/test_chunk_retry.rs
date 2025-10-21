use crate::common::*;

/// Unknown chunk should return 404 (per OpenAPI)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t023_contract_post_chunk_retry_unknown_returns_404() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let chunk_id = uuid::Uuid::new_v4();
    let url = format!("{}/chunks/{}/retry", api(&srv.base_url), chunk_id);
    let resp = reqwest::Client::new().post(&url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 404, "unknown chunk must be 404");
}
