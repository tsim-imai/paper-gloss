use crate::common::*;

use crate::common as common;

/// Merge must require confirmed=true
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t076_merge_missing_confirmation_returns_400() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let url = format!("{}/terms/merge", api(&srv.base_url));
    let body = serde_json::json!({
        "source_id": uuid::Uuid::new_v4().to_string(),
        "target_id": uuid::Uuid::new_v4().to_string(),
        "confirmed": false
    });
    let resp = reqwest::Client::new().post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 400, "merge without confirmation must be 400");
}

/// Merge with same IDs must be 400
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t076_merge_same_ids_returns_400() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let url = format!("{}/terms/merge", api(&srv.base_url));
    let id = uuid::Uuid::new_v4().to_string();
    let body = serde_json::json!({
        "source_id": id,
        "target_id": id,
        "confirmed": true
    });
    let resp = reqwest::Client::new().post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 400, "merge with same IDs must be 400");
}
