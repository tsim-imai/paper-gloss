use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t020_contract_paper_detail_returns_200_for_existing() {
    let srv = TestServer::spawn().expect("failed to start test server");
    // Use a dummy UUID; once implementation exists, test should create a paper first.
    let paper_id = uuid::Uuid::new_v4();
    let url = format!("{}/papers/{}", api(&srv.base_url), paper_id);
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
}
