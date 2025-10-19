use crate::common::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_contract_paper_import_requires_multipart_and_returns_201() {
    // Start server
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));

    // Per OpenAPI, request is multipart/form-data with either file or (url,title)
    // Use file path to avoid network dependency.
    let pdf_bytes = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
    let part = reqwest::multipart::Part::bytes(pdf_bytes)
        .file_name("sample.pdf")
        .mime_str("application/pdf").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Sample Paper");

    let resp = client.post(&url).multipart(form).send().await.unwrap();

    // Contract expectation (will FAIL until implemented): 201 Created
    assert_eq!(resp.status().as_u16(), 201, "expected 201 Created per OpenAPI");

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.get("paper_id").is_some(), "missing paper_id in response");
}

/// Non-arXiv URL should be rejected with 400
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_non_arxiv_url_returns_400() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));
    let form = reqwest::multipart::Form::new()
        .text("url", "https://example.com/abs/1234.5678")
        .text("title", "Sample");
    let resp = client.post(&url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 400, "non-arXiv URL must be 400");
}

/// URL import without title must be 400
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_url_missing_title_returns_400() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));
    let form = reqwest::multipart::Form::new()
        .text("url", "https://arxiv.org/abs/2212.14578");
    let resp = client.post(&url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 400, "missing title must be 400");
}

/// File upload with invalid content-type should be 422 (per spec)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_file_invalid_content_type_returns_422() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));
    let bytes = b"NOT_A_PDF".to_vec();
    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name("notpdf.txt")
        .mime_str("text/plain").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Invalid");
    let resp = client.post(&url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 422, "invalid file must be 422");
}

/// File upload exceeding 100MB should be 422
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_file_too_large_returns_422() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));
    // 100 MB + 1 byte
    let oversized = vec![0u8; 100 * 1024 * 1024 + 1];
    let part = reqwest::multipart::Part::bytes(oversized)
        .file_name("big.pdf")
        .mime_str("application/pdf").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Too Big");
    let resp = client.post(&url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 422, "oversized file must be 422");
}
