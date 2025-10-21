use crate::common::*;
use httpmock::Method::POST;
use httpmock::MockServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t077_contract_post_term_define_returns_200_with_openai_mock() {
    // Mock OpenAI-compatible /v1/chat/completions
    let llm = MockServer::start_async().await;
    llm.mock_async(|when, then| {
        when.method(POST).path("/v1/chat/completions");
        then.status(200).json_body(serde_json::json!({
            "id": "chatcmpl-def-1",
            "object": "chat.completion",
            "created": 0,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "定義テキスト"},
                "finish_reason": "stop"
            }]
        }));
    }).await;

    // Spawn API with LLM env
    let extra_env = vec![
        ("AI_API_BASE".to_string(), format!("{}/v1", llm.base_url())),
        ("AI_API_KEY".to_string(), "sk-test".to_string()),
    ];
    let srv = TestServer::spawn_with_env(extra_env).await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Create a term first
    let create_url = format!("{}/terms", api(&srv.base_url));
    let created = client.post(&create_url).json(&serde_json::json!({
        "lemma_en": "transformer",
        "lemma_ja": "トランスフォーマー"
    })).send().await.unwrap();
    assert_eq!(created.status().as_u16(), 201);
    let id = created.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().to_string();

    // Define
    let url = format!("{}/terms/{}/define", api(&srv.base_url), id);
    let resp = client.post(&url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
}
