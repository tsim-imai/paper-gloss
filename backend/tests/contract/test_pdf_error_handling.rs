use crate::common::*;
use serde_json::json;

/// T046: Test PDF error handling - empty file (FR-005)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t046_pdf_empty_file_returns_422() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    let url = format!("{}/papers/import", api(&srv.base_url));

    // Create empty file
    let form = reqwest::multipart::Form::new()
        .text("title", "Empty PDF Test")
        .part("file", reqwest::multipart::Part::bytes(vec![])
            .file_name("empty.pdf")
            .mime_str("application/pdf")
            .unwrap());

    let resp = client.post(&url)
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 422, "Empty file should return 422");

    let error: serde_json::Value = resp.json().await.unwrap();
    println!("Error response: {:?}", error);
    let error_msg = error["message"].as_str().unwrap_or(error["error"].as_str().unwrap_or(""));
    assert!(error_msg.to_lowercase().contains("empty"),
            "Error message should mention empty file, got: {}", error_msg);
}

/// T046: Test PDF error handling - invalid format (FR-005)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t046_pdf_invalid_format_returns_422() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    let url = format!("{}/papers/import", api(&srv.base_url));

    // Create file with invalid content (not PDF)
    let form = reqwest::multipart::Form::new()
        .text("title", "Invalid PDF Test")
        .part("file", reqwest::multipart::Part::bytes(b"This is not a PDF file".to_vec())
            .file_name("invalid.pdf")
            .mime_str("application/pdf")
            .unwrap());

    let resp = client.post(&url)
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 422, "Invalid format should return 422");

    let error: serde_json::Value = resp.json().await.unwrap();
    println!("Error response: {:?}", error);
    let error_msg = error["message"].as_str().unwrap_or(error["error"].as_str().unwrap_or(""));
    assert!(error_msg.contains("Invalid file format") || error_msg.contains("PDF"),
            "Error message should mention invalid format, got: {}", error_msg);
}

/// T046: Test PDF error handling - file too large (FR-005)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t046_pdf_too_large_returns_422() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    let url = format!("{}/papers/import", api(&srv.base_url));

    // Create a fake PDF that's too large (>100MB)
    // We'll create a valid PDF header but with huge content
    let mut large_file = b"%PDF-1.4\n".to_vec();
    // Add 101MB of data
    large_file.extend(vec![0u8; 101 * 1024 * 1024]);

    let form = reqwest::multipart::Form::new()
        .text("title", "Large PDF Test")
        .part("file", reqwest::multipart::Part::bytes(large_file)
            .file_name("large.pdf")
            .mime_str("application/pdf")
            .unwrap());

    let resp = client.post(&url)
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 422, "Too large file should return 422");

    let error: serde_json::Value = resp.json().await.unwrap();
    println!("Error response: {:?}", error);
    let error_msg = error["message"].as_str().unwrap_or(error["error"].as_str().unwrap_or(""));
    assert!(error_msg.to_lowercase().contains("too large") || error_msg.to_lowercase().contains("max"),
            "Error message should mention file too large, got: {}", error_msg);
}

/// T046: Test PDF error handling - corrupted but valid header (FR-007: partial processing)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t046_pdf_corrupted_but_valid_header() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    let url = format!("{}/papers/import", api(&srv.base_url));

    // Create a PDF with valid header but corrupted content
    let corrupted_pdf = b"%PDF-1.4\n%%corrupted content here\ngarbage data".to_vec();

    let form = reqwest::multipart::Form::new()
        .text("title", "Corrupted PDF Test")
        .part("file", reqwest::multipart::Part::bytes(corrupted_pdf)
            .file_name("corrupted.pdf")
            .mime_str("application/pdf")
            .unwrap());

    let resp = client.post(&url)
        .multipart(form)
        .send()
        .await
        .unwrap();

    // Should accept the file (201) but processing may fail later
    assert_eq!(resp.status().as_u16(), 201, "Should accept PDF with valid header");

    let result: serde_json::Value = resp.json().await.unwrap();
    let paper_id = result["paper_id"].as_str().unwrap();

    // Wait for processing
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Check status - should be failed or have warnings
    let status_url = format!("{}/papers/{}/status", api(&srv.base_url), paper_id);
    let status_resp = client.get(&status_url).send().await.unwrap();

    if status_resp.status().as_u16() == 200 {
        let status: serde_json::Value = status_resp.json().await.unwrap();
        // Processing might fail or produce partial results
        println!("Paper status after corrupted PDF: {:?}", status);
    }
}

/// T046: Test missing both file and URL (FR-005)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t046_missing_file_and_url_returns_400() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    let url = format!("{}/papers/import", api(&srv.base_url));

    // Send only title, no file or URL
    let form = reqwest::multipart::Form::new()
        .text("title", "No Content Test");

    let resp = client.post(&url)
        .multipart(form)
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 400, "Missing file and URL should return 400");

    let error: serde_json::Value = resp.json().await.unwrap();
    let error_msg = error["message"].as_str().unwrap_or(error["error"].as_str().unwrap_or(""));
    assert!(error_msg.to_lowercase().contains("file") ||
            error_msg.to_lowercase().contains("url"),
            "Error message should mention missing file or URL, got: {}", error_msg);
}