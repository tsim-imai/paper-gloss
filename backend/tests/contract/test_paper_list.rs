use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t019_contract_paper_list_returns_200_with_pagination() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let url = format!("{}/papers?page=1&limit=20", api(&srv.base_url));
    let resp = reqwest::get(&url).await.unwrap();

    // Per OpenAPI: 200 with { papers, total, page, limit }
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
    let json: serde_json::Value = resp.json().await.unwrap();
    for k in ["papers", "total", "page", "limit"].iter() {
        assert!(json.get(*k).is_some(), "missing key {}", k);
    }
}
