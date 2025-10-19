use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t021_contract_get_translation_returns_200_with_chunks() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    // Ensure a paper exists
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    let pdf_bytes = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
    let part = reqwest::multipart::Part::bytes(pdf_bytes)
        .file_name("sample.pdf")
        .mime_str("application/pdf").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Sample Paper");
    let resp = reqwest::Client::new().post(&import_url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201, "expected 201 Created per OpenAPI");
    let paper_id = resp.json::<serde_json::Value>().await.unwrap()["paper_id"].as_str().unwrap().to_string();
    let url = format!("{}/papers/{}/translation", api(&srv.base_url), paper_id);
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "expected 200 OK per OpenAPI");
    let json: serde_json::Value = resp.json().await.unwrap();
    assert!(json.get("chunks").is_some(), "missing chunks array");
}
