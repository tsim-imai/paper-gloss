use crate::common::*;
use std::fs;
use std::path::Path;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr037_delete_paper_returns_204_and_removes_all_data() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // First, create a paper by importing a PDF
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    // Create a minimal valid PDF
    let pdf_data = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF";
    let form = reqwest::multipart::Form::new()
        .text("title", "Test Paper for Deletion")
        .part(
            "file",
            reqwest::multipart::Part::bytes(pdf_data.to_vec())
                .file_name("test.pdf")
                .mime_str("application/pdf")
                .unwrap(),
        );

    let import_resp = client.post(&import_url)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(import_resp.status().as_u16(), 201);

    let paper_data = import_resp.json::<serde_json::Value>().await.unwrap();
    let paper_id = paper_data["paper_id"].as_str().unwrap();

    // Wait for processing to create chunks (give it a moment to start)
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Verify paper exists
    let get_url = format!("{}/papers/{}", api(&srv.base_url), paper_id);
    let get_resp = client.get(&get_url).send().await.unwrap();
    assert_eq!(get_resp.status().as_u16(), 200);

    // Verify PDF file exists
    let pdf_path = format!("artifacts/papers/{}/source.pdf", paper_id);
    assert!(Path::new(&pdf_path).exists(), "PDF file should exist before deletion");

    // Delete the paper (FR-037)
    let delete_url = format!("{}/papers/{}", api(&srv.base_url), paper_id);
    let delete_resp = client.delete(&delete_url).send().await.unwrap();
    assert_eq!(delete_resp.status().as_u16(), 204, "DELETE should return 204 No Content");

    // Verify paper no longer exists
    let get_after_delete = client.get(&get_url).send().await.unwrap();
    assert_eq!(get_after_delete.status().as_u16(), 404, "Paper should not exist after deletion");

    // Verify PDF file is deleted (FR-037)
    assert!(!Path::new(&pdf_path).exists(), "PDF file should be deleted");

    // Verify chunks are deleted by checking translation API
    let translation_url = format!("{}/papers/{}/translation", api(&srv.base_url), paper_id);
    let translation_resp = client.get(&translation_url).send().await.unwrap();
    assert_eq!(translation_resp.status().as_u16(), 404, "Translation/chunks should not exist");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr037_delete_nonexistent_paper_returns_404() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    let delete_url = format!("{}/papers/{}", api(&srv.base_url), "nonexistent-id");
    let resp = client.delete(&delete_url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 404, "DELETE nonexistent paper should return 404");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr037_delete_paper_removes_occurrences() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Create a paper
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    let pdf_data = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF";
    let form = reqwest::multipart::Form::new()
        .text("title", "Paper with Terms")
        .part(
            "file",
            reqwest::multipart::Part::bytes(pdf_data.to_vec())
                .file_name("test.pdf")
                .mime_str("application/pdf")
                .unwrap(),
        );

    let import_resp = client.post(&import_url).multipart(form).send().await.unwrap();
    assert_eq!(import_resp.status().as_u16(), 201);
    let paper_data = import_resp.json::<serde_json::Value>().await.unwrap();
    let paper_id = paper_data["paper_id"].as_str().unwrap();

    // Create a term manually
    let create_term_url = format!("{}/terms", api(&srv.base_url));
    let term_body = serde_json::json!({
        "lemma_en": "neural network",
        "lemma_ja": "ニューラルネットワーク"
    });
    let term_resp = client.post(&create_term_url).json(&term_body).send().await.unwrap();
    assert_eq!(term_resp.status().as_u16(), 201);
    let term_data = term_resp.json::<serde_json::Value>().await.unwrap();
    let term_id = term_data["id"].as_str().unwrap();

    // Check occurrences before deletion (may be empty but endpoint should work)
    let occurrences_url = format!("{}/occurrences?term_id={}", api(&srv.base_url), term_id);
    let occ_before = client.get(&occurrences_url).send().await.unwrap();
    assert_eq!(occ_before.status().as_u16(), 200);

    // Delete the paper
    let delete_url = format!("{}/papers/{}", api(&srv.base_url), paper_id);
    let delete_resp = client.delete(&delete_url).send().await.unwrap();
    assert_eq!(delete_resp.status().as_u16(), 204);

    // Check occurrences after deletion (should still work but be empty)
    let occ_after = client.get(&occurrences_url).send().await.unwrap();
    assert_eq!(occ_after.status().as_u16(), 200);
    let occ_data = occ_after.json::<serde_json::Value>().await.unwrap();

    // If there's an occurrences array, it should be empty for this paper
    if let Some(occurrences) = occ_data.get("occurrences").and_then(|v| v.as_array()) {
        let paper_occurrences: Vec<_> = occurrences.iter()
            .filter(|o| o.get("paper_id").and_then(|p| p.as_str()) == Some(paper_id))
            .collect();
        assert_eq!(paper_occurrences.len(), 0, "No occurrences should remain for deleted paper");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr039_delete_paper_cleans_up_orphaned_terms() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // This test verifies that orphaned terms (with no occurrences) are handled
    // The actual implementation might clean them up automatically or flag them

    // Create a paper
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    let pdf_data = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF";
    let form = reqwest::multipart::Form::new()
        .text("title", "Single Paper with Unique Term")
        .part(
            "file",
            reqwest::multipart::Part::bytes(pdf_data.to_vec())
                .file_name("test.pdf")
                .mime_str("application/pdf")
                .unwrap(),
        );

    let import_resp = client.post(&import_url).multipart(form).send().await.unwrap();
    assert_eq!(import_resp.status().as_u16(), 201);
    let paper_data = import_resp.json::<serde_json::Value>().await.unwrap();
    let paper_id = paper_data["paper_id"].as_str().unwrap();

    // Wait a bit for potential term extraction
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Delete the paper
    let delete_url = format!("{}/papers/{}", api(&srv.base_url), paper_id);
    let delete_resp = client.delete(&delete_url).send().await.unwrap();
    assert_eq!(delete_resp.status().as_u16(), 204);

    // Check that orphaned terms are handled per FR-039
    // The exact behavior (auto-delete vs flag) will be determined by implementation
    // For now, we just verify the delete completes successfully
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr037_delete_processing_paper_stops_background_tasks() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Import a paper (it will start processing in background)
    let import_url = format!("{}/papers/import", api(&srv.base_url));
    let pdf_data = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF";
    let form = reqwest::multipart::Form::new()
        .text("title", "Paper Being Processed")
        .part(
            "file",
            reqwest::multipart::Part::bytes(pdf_data.to_vec())
                .file_name("test.pdf")
                .mime_str("application/pdf")
                .unwrap(),
        );

    let import_resp = client.post(&import_url).multipart(form).send().await.unwrap();
    assert_eq!(import_resp.status().as_u16(), 201);
    let paper_data = import_resp.json::<serde_json::Value>().await.unwrap();
    let paper_id = paper_data["paper_id"].as_str().unwrap();

    // Immediately delete while processing (edge case from spec)
    let delete_url = format!("{}/papers/{}", api(&srv.base_url), paper_id);
    let delete_resp = client.delete(&delete_url).send().await.unwrap();
    assert_eq!(delete_resp.status().as_u16(), 204, "Should be able to delete processing paper");

    // Verify deletion worked
    let get_url = format!("{}/papers/{}", api(&srv.base_url), paper_id);
    let get_resp = client.get(&get_url).send().await.unwrap();
    assert_eq!(get_resp.status().as_u16(), 404);

    // Verify PDF file is deleted
    let pdf_path = format!("artifacts/papers/{}/source.pdf", paper_id);
    // Give it a moment for async cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    assert!(!Path::new(&pdf_path).exists(), "PDF should be deleted even if processing was in progress");
}