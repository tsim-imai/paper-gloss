use crate::common::*;

/// POST /terms/{id}/define for unknown term should return 404 Not Found
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t077_contract_post_term_define_returns_404_for_unknown_term() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Random UUID that won't exist in the fresh temp DB
    let unknown_id = uuid::Uuid::new_v4().to_string();
    let url = format!("{}/terms/{}/define", api(&srv.base_url), unknown_id);

    let resp = client.post(&url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 404, "expected 404 Not Found for unknown term id");
}

