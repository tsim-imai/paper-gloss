use crate::common::*;

/// US3: Glossary CRUD workflow
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t078_integration_glossary_crud_workflow() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let client = reqwest::Client::new();

    // Create term
    let create_url = format!("{}/terms", api(&srv.base_url));
    let create_body = serde_json::json!({
        "lemma_en": "gradient descent",
        "lemma_ja": "勾配降下法"
    });
    let resp = client.post(&create_url).json(&create_body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201, "create term should return 201");
    let term: serde_json::Value = resp.json().await.unwrap();
    let id = term["id"].as_str().unwrap().to_string();

    // Update term
    let update_url = format!("{}/terms/{}", api(&srv.base_url), id);
    let resp = client.patch(&update_url).json(&serde_json::json!({"note":"test"})).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "update should return 200");

    // List terms
    let list_url = format!("{}/terms?q=gradient", api(&srv.base_url));
    let resp = reqwest::get(&list_url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "list should return 200");

    // Delete term
    let delete_url = format!("{}/terms/{}", api(&srv.base_url), id);
    let resp = client.delete(&delete_url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 204, "delete should return 204");
}
