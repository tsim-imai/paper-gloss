use crate::common::*;

/// Unknown paper id should return 404
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t031_paper_status_unknown_returns_404() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let unknown = uuid::Uuid::new_v4().to_string();
    let url = format!("{}/papers/{}/status", api(&srv.base_url), unknown);
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 404, "unknown paper must be 404");
}

/// Existing paper should return 200 with progress structure
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t031_paper_status_returns_progress_structure() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Import a minimal PDF to create a paper
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    let pdf_bytes = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
    let part = reqwest::multipart::Part::bytes(pdf_bytes)
        .file_name("sample.pdf")
        .mime_str("application/pdf").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Status Test Paper");
    let resp = client.post(&import_url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201, "import should return 201");
    let body: serde_json::Value = resp.json().await.unwrap();
    let paper_id = body["paper_id"].as_str().unwrap();

    // Query status
    let status_url = format!("{}/papers/{}/status", api(&srv.base_url), paper_id);
    let status = reqwest::get(&status_url).await.unwrap();
    assert_eq!(status.status().as_u16(), 200);
    let json: serde_json::Value = status.json().await.unwrap();

    // Basic structure assertions
    assert_eq!(json["paper_id"].as_str().unwrap(), paper_id);
    let _status_str = json["status"].as_str().expect("status must be string");

    let progress = json.get("progress").expect("missing progress");
    assert!(progress["extraction"].as_str().is_some());

    let trans = &progress["translation"];
    let total = trans["total_chunks"].as_i64().unwrap_or(-1);
    let completed = trans["completed_chunks"].as_i64().unwrap_or(-1);
    let failed = trans["failed_chunks"].as_i64().unwrap_or(-1);
    assert!(total >= 0 && completed >= 0 && failed >= 0);
    assert!(completed + failed <= total || total == 0);

    let defs = &progress["definitions"];
    let total_terms = defs["total_terms"].as_i64().unwrap_or(-1);
    let completed_defs = defs["completed_definitions"].as_i64().unwrap_or(-1);
    assert!(total_terms >= 0 && completed_defs >= 0);
}

