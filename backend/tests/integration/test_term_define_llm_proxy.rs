use crate::common::*;
use httpmock::Method::POST;
use httpmock::MockServer;

/// /api 経由で LLM モックを叩く結合テスト（POST /terms/{id}/define）
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t077_integration_define_uses_openai_compat_bridge() {
    // Start mock OpenAI-compatible server
    let llm = MockServer::start_async().await;

    // One completion call returning definition text
    llm.mock_async(|when, then| {
        when.method(POST).path("/v1/chat/completions");
        then.status(200).json_body(serde_json::json!({
            "id": "chatcmpl-def-1",
            "object": "chat.completion",
            "created": 0,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "日本語の定義テキスト"},
                "finish_reason": "stop"
            }]
        }));
    }).await;

    // Spawn backend server wired to the mock LLM
    let extra_env = vec![
        ("AI_API_BASE".to_string(), format!("{}/v1", llm.base_url())),
        ("AI_API_KEY".to_string(), "sk-test".to_string()),
    ];
    let srv = TestServer::spawn_with_env(extra_env).await.expect("start server");
    let client = reqwest::Client::new();

    // Create a term first
    let create_url = format!("{}/terms", api(&srv.base_url));
    let body = serde_json::json!({
        "lemma_en": "knowledge distillation",
        "lemma_ja": "知識蒸留"
    });
    let resp = client.post(&create_url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201);
    let term: serde_json::Value = resp.json().await.unwrap();
    let id = term["id"].as_str().unwrap();

    // Call define (should hit mock LLM and persist)
    let define_url = format!("{}/terms/{}/define", api(&srv.base_url), id);
    let resp = client.post(&define_url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["term_id"].as_str().unwrap(), id);
    assert_eq!(json["definition"].as_str().unwrap(), "日本語の定義テキスト");
}

/// LLM 側 500 → アプリは 503 を返すこと
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t077_integration_define_returns_503_on_llm_failure() {
    let llm = MockServer::start_async().await;
    llm.mock_async(|when, then| {
        when.method(POST).path("/v1/chat/completions");
        then.status(500).json_body(serde_json::json!({
            "error": {"message": "server error", "type": "server_error"}
        }));
    }).await;

    let extra_env = vec![
        ("AI_API_BASE".to_string(), format!("{}/v1", llm.base_url())),
        ("AI_API_KEY".to_string(), "sk-test".to_string()),
    ];
    let srv = TestServer::spawn_with_env(extra_env).await.expect("start server");
    let client = reqwest::Client::new();

    // Create a term
    let create_url = format!("{}/terms", api(&srv.base_url));
    let body = serde_json::json!({
        "lemma_en": "transformer",
        "lemma_ja": "トランスフォーマー"
    });
    let resp = client.post(&create_url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201);
    let term: serde_json::Value = resp.json().await.unwrap();
    let id = term["id"].as_str().unwrap();

    // Define should map upstream failure to 503
    let define_url = format!("{}/terms/{}/define", api(&srv.base_url), id);
    let resp = client.post(&define_url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 503);
}

