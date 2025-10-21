use crate::common::*;

/// GET /papers/{id}/translation returns chunks ordered by `chunk_index` asc
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t022_contract_translation_chunks_are_sorted_by_index() {
    let srv = TestServer::spawn().await.expect("failed to start test server");

    // Import a minimal PDF (processing may yield zero chunks; order must still be non-decreasing)
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    let pdf_bytes = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
    let part = reqwest::multipart::Part::bytes(pdf_bytes)
        .file_name("sample.pdf")
        .mime_str("application/pdf").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Sorted Chunks Paper");
    let resp = reqwest::Client::new().post(&import_url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201);
    let paper_id = resp.json::<serde_json::Value>().await.unwrap()["paper_id"].as_str().unwrap().to_string();

    // Fetch translation and verify order of chunk_index
    let url = format!("{}/papers/{}/translation", api(&srv.base_url), paper_id);
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();

    let indices: Vec<i32> = json["chunks"].as_array()
        .unwrap()
        .iter()
        .map(|c| c["chunk_index"].as_i64().unwrap_or(0) as i32)
        .collect();

    let mut sorted = indices.clone();
    sorted.sort();
    assert_eq!(indices, sorted, "chunks must be returned in ascending index order");
}

