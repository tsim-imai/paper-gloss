use crate::common::*;

use crate::common as common;

/// Invalid enum values for lang/sort should be 400
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t072_terms_invalid_lang_sort_returns_400() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let url = format!("{}/terms?lang=xx&sort=bad", api(&srv.base_url));
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 400, "invalid enum values must be 400");
}

/// Invalid pagination values should be 400
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t072_terms_invalid_pagination_returns_400() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let url = format!("{}/terms?page=0&limit=0", api(&srv.base_url));
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 400, "invalid pagination must be 400");
}
