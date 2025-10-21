use crate::common::*;

/// Zero-byte PDF should be 422 (fails magic bytes check)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_zero_byte_pdf_returns_422() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));

    let part = reqwest::multipart::Part::bytes(Vec::new())
        .file_name("empty.pdf")
        .mime_str("application/pdf").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Empty PDF");

    let resp = client.post(&url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 422, "zero-byte file must be 422");
}

/// Missing both file and url fields must be 400 Bad Request
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_missing_both_file_and_url_returns_400() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));

    // Empty multipart form
    let form = reqwest::multipart::Form::new();
    let resp = client.post(&url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 400, "missing file/url must be 400");
}

/// application/pdf MIMEだが先頭が%PDFでない → 422
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_pdf_mime_but_invalid_magic_returns_422() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));

    let bogus_pdf = b"NOT%PDF".to_vec();
    let part = reqwest::multipart::Part::bytes(bogus_pdf)
        .file_name("bogus.pdf")
        .mime_str("application/pdf").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Bogus PDF");

    let resp = client.post(&url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 422, "invalid magic must be 422");
}

/// 成功時に Location ヘッダが /api/papers/{id} 形式で返る
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t018_success_sets_location_header() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();
    let url = format!("{}/papers/import", api(&srv.base_url));

    let pdf_bytes = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
    let part = reqwest::multipart::Part::bytes(pdf_bytes)
        .file_name("ok.pdf")
        .mime_str("application/pdf").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Has Location Header");

    let resp = client.post(&url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201);

    let loc = resp.headers().get("Location").and_then(|v| v.to_str().ok()).unwrap_or("");
    assert!(loc.starts_with("/api/papers/"), "Location header must start with /api/papers/, got: {}", loc);
}
