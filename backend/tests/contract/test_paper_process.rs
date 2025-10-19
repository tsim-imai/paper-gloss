use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t022_contract_post_paper_process_returns_202() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    // Create a paper via file import
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
    let url = format!("{}/papers/{}/process", api(&srv.base_url), paper_id);
    let resp = reqwest::Client::new().post(&url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 202, "expected 202 Accepted per OpenAPI");
}
