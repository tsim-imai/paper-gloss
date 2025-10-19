use crate::common::*;

/// US1 end-to-end: import → process → get translation
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t024_integration_pdf_upload_to_translation() {
    let srv = TestServer::spawn().expect("failed to start test server");
    let client = reqwest::Client::new();

    // 1) Import by URL
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    let form = reqwest::multipart::Form::new()
        .text("url", "https://arxiv.org/abs/2212.14578")
        .text("title", "Constitutional AI: Harmlessness from AI Feedback");
    let resp = client.post(&import_url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201, "import should return 201");
    let body: serde_json::Value = resp.json().await.unwrap();
    let paper_id = body.get("paper_id").unwrap().as_str().unwrap().to_string();

    // 2) Trigger processing
    let process_url = format!("{}/papers/{}/process", api(&srv.base_url), paper_id);
    let resp = client.post(&process_url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 202, "process should return 202");

    // 3) Fetch translation (polling simplified)
    let trans_url = format!("{}/papers/{}/translation", api(&srv.base_url), paper_id);
    let trans = reqwest::get(&trans_url).await.unwrap();
    assert_eq!(trans.status().as_u16(), 200, "translation should return 200");
}
