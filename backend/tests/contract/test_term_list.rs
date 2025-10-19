use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t072_contract_get_terms_returns_200_with_pagination() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let url = format!("{}/terms?lang=both&sort=alphabetical&page=1&limit=50", api(&srv.base_url));
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
}
