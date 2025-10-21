use crate::common::*;
use httpmock::{Method::POST, MockServer};
use sqlx::Row;

/// Happy path: create a real chunk in the test DB, call /chunks/{id}/retry,
/// expect 202 Accepted, then observe retry_count increment via translation API.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t023_contract_post_chunk_retry_returns_202_and_increments_retry_count() {
    // Mock LLM endpoint to ensure retry succeeds quickly
    let llm = MockServer::start_async().await;
    llm.mock_async(|when, then| {
        when.method(POST).path("/v1/chat/completions");
        then.status(200).json_body(serde_json::json!({
            "id": "chatcmpl-retry",
            "object": "chat.completion",
            "created": 0,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "再翻訳結果"},
                "finish_reason": "stop"
            }]
        }));
    }).await;

    // Start server wired to mock LLM
    let extra_env = vec![
        ("AI_API_BASE".to_string(), format!("{}/v1", llm.base_url())),
        ("AI_API_KEY".to_string(), "sk-test".to_string()),
    ];
    let srv = TestServer::spawn_with_env(extra_env).await.expect("server");

    // Insert a paper and a chunk directly into the same SQLite DB
    let pool = srv.connect_db().await.expect("db connect");
    let paper_id = uuid::Uuid::new_v4().to_string();
    let chunk_id = uuid::Uuid::new_v4().to_string();
    let content_hash = uuid::Uuid::new_v4().to_string();

    // Insert paper
    sqlx::query(
        r#"
        INSERT INTO papers (id, title, source_url, file_path, status, created_at, updated_at)
        VALUES (?1, ?2, NULL, ?3, 'processing', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&paper_id)
    .bind("Test Paper for Retry")
    .bind(format!("artifacts/papers/{}/source.pdf", &paper_id))
    .execute(&pool)
    .await
    .expect("insert paper");

    // Insert chunk (failed state to make semantics clear)
    sqlx::query(
        r#"
        INSERT INTO chunks (id, paper_id, index_, src_text, trans_html, content_hash, token_count, status, retry_count, error_message, created_at, updated_at)
        VALUES (?1, ?2, 0, ?3, NULL, ?4, NULL, 'failed', 0, 'LLM error', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&chunk_id)
    .bind(&paper_id)
    .bind("This is the source text for retry")
    .bind(&content_hash)
    .execute(&pool)
    .await
    .expect("insert chunk");

    // Call retry endpoint
    let url = format!("{}/chunks/{}/retry", api(&srv.base_url), &chunk_id);
    let resp = reqwest::Client::new().post(&url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 202, "retry should be accepted with 202");

    // Poll translation API to observe retry_count increment
    let trans_url = format!("{}/papers/{}/translation", api(&srv.base_url), &paper_id);
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(5);
    let mut observed = false;
    while start.elapsed() < timeout {
        let r = reqwest::get(&trans_url).await.unwrap();
        assert_eq!(r.status().as_u16(), 200);
        let json: serde_json::Value = r.json().await.unwrap();
        if let Some(arr) = json["chunks"].as_array() {
            if let Some(me) = arr.iter().find(|c| c["id"].as_str() == Some(&chunk_id)) {
                let cnt = me["retry_count"].as_i64().unwrap_or(0);
                if cnt >= 1 { observed = true; break; }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    assert!(observed, "expected retry_count to increment within timeout");
}

