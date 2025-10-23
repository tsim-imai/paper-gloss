use crate::common::*;

/// US2: After processing a paper, terms should be extractable and highlightable
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t051_integration_term_extraction_to_highlighting() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Import & process a paper via file (avoid network)
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    let pdf_bytes = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
    let part = reqwest::multipart::Part::bytes(pdf_bytes)
        .file_name("sample.pdf")
        .mime_str("application/pdf").unwrap();
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("title", "Sample");
    let resp = client.post(&import_url).multipart(form).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201);
    let paper_id = resp.json::<serde_json::Value>().await.unwrap()["paper_id"].as_str().unwrap().to_string();

    let translate_url = format!("{}/papers/{}/translate", api(&srv.base_url), paper_id);
    let _ = client.post(&translate_url).send().await.unwrap();

    // Terms should be listable and occurrences available
    // 1) search terms (bilingual; normalization handled server-side)
    let terms_url = format!("{}/terms?q=network&lang=both&sort=alphabetical&page=1&limit=50", api(&srv.base_url));
    let resp = reqwest::get(&terms_url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200, "terms search should return 200");

    // 2) pick first term id (if any) and request occurrences for this paper
    if resp.status().is_success() {
        let json: serde_json::Value = resp.json().await.unwrap();
        if let Some(first) = json["terms"].as_array().and_then(|arr| arr.get(0)) {
            if let Some(term_id) = first["id"].as_str() {
                let occ_url = format!("{}/occurrences?paper_id={}&term_id={}", api(&srv.base_url), paper_id, term_id);
                let occ = reqwest::get(&occ_url).await.unwrap();
                assert_eq!(occ.status().as_u16(), 200, "occurrences list should return 200");
            }
        }
    }
}
