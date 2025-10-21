use crate::common::*;
use serde_json::json;

/// FR-034: データ永続性チェック - サーバー再起動後もデータが保持されることを確認
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr034_data_persists_across_server_restarts() {
    // Create a persistent DB file
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db_path = tmp.into_temp_path().to_path_buf();

    // First server instance
    let term_id: String;
    {
        let srv1 = TestServer::spawn_with_db_path(db_path.clone())
            .await
            .expect("failed to start first server instance");
        let client = reqwest::Client::new();

        // Create a term
        let create_url = format!("{}/terms", api(&srv1.base_url));
        let body = json!({
            "lemma_en": "persistent term",
            "lemma_ja": "永続的な用語",
            "note": "This term should survive server restart"
        });

        let resp = client.post(&create_url).json(&body).send().await.unwrap();
        assert_eq!(resp.status().as_u16(), 201);

        let created_term: serde_json::Value = resp.json().await.unwrap();
        term_id = created_term["id"].as_str().unwrap().to_string();

        // Verify the term exists
        let get_url = format!("{}/terms/{}", api(&srv1.base_url), term_id);
        let resp = client.get(&get_url).send().await.unwrap();
        assert_eq!(resp.status().as_u16(), 200);

        // Stop the first server
        srv1.stop();
    }

    // Wait a bit to ensure the server has fully stopped
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Second server instance with the same database
    {
        let srv2 = TestServer::spawn_with_db_path(db_path.clone())
            .await
            .expect("failed to start second server instance");
        let client = reqwest::Client::new();

        // Verify the term still exists after restart
        let get_url = format!("{}/terms/{}", api(&srv2.base_url), term_id);
        let resp = client.get(&get_url).send().await.unwrap();
        assert_eq!(resp.status().as_u16(), 200, "Term should persist after server restart");

        let retrieved_term: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(retrieved_term["lemma_en"], "persistent term");
        assert_eq!(retrieved_term["lemma_ja"], "永続的な用語");
        assert_eq!(retrieved_term["note"], "This term should survive server restart");

        // Stop the second server
        srv2.stop();
    }
}

/// FR-034: Paper data persistence test
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr034_papers_persist_across_restarts() {
    // Create a persistent DB file
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db_path = tmp.into_temp_path().to_path_buf();

    let paper_id: String;
    {
        let srv1 = TestServer::spawn_with_db_path(db_path.clone())
            .await
            .expect("failed to start first server instance");
        let client = reqwest::Client::new();

        // Create a paper via URL import
        let import_url = format!("{}/papers/import", api(&srv1.base_url));
        let body = json!({
            "url": "https://arxiv.org/abs/1234.5678",
            "title": "Test Paper for Persistence"
        });

        let resp = client.post(&import_url).json(&body).send().await.unwrap();
        // Note: May fail if arXiv URL is invalid, but we're testing persistence not import
        if resp.status().as_u16() == 201 {
            let created_paper: serde_json::Value = resp.json().await.unwrap();
            paper_id = created_paper["id"].as_str().unwrap().to_string();
        } else {
            // Create a paper directly using the database if import fails
            let pool = srv1.connect_db().await.unwrap();
            let id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO papers (id, title, source_url, file_path, status, created_at, updated_at)
                VALUES (?, ?, ?, ?, 'pending', datetime('now'), datetime('now'))
                "#
            )
            .bind(&id)
            .bind("Test Paper for Persistence")
            .bind("https://example.com/test.pdf")
            .bind("/tmp/test.pdf")
            .execute(&pool)
            .await
            .unwrap();
            paper_id = id;
        }

        srv1.stop();
    }

    // Wait for server to fully stop
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Restart and verify
    {
        let srv2 = TestServer::spawn_with_db_path(db_path.clone())
            .await
            .expect("failed to start second server instance");
        let client = reqwest::Client::new();

        // Check if paper still exists
        let list_url = format!("{}/papers", api(&srv2.base_url));
        let resp = client.get(&list_url).send().await.unwrap();
        assert_eq!(resp.status().as_u16(), 200);

        let papers: serde_json::Value = resp.json().await.unwrap();
        let items = papers["items"].as_array().unwrap();
        assert!(!items.is_empty(), "Papers should persist after restart");

        // Find our specific paper
        // Debug output to see what's in the database
        println!("Looking for paper_id: {}", paper_id);
        println!("Found {} papers in database", items.len());
        for item in items {
            println!("Paper in DB: id={}, title={}",
                     item["id"].as_str().unwrap_or("unknown"),
                     item["title"].as_str().unwrap_or("unknown"));
        }

        let found = items.iter().any(|p| p["id"] == paper_id);
        assert!(found, "Specific paper {} should persist after restart", paper_id);

        srv2.stop();
    }
}

/// FR-034: Multiple restarts persistence test
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr034_data_persists_through_multiple_restarts() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let db_path = tmp.into_temp_path().to_path_buf();

    let mut term_ids = Vec::new();

    // Create terms across multiple server sessions
    for i in 0..3 {
        let srv = TestServer::spawn_with_db_path(db_path.clone())
            .await
            .expect(&format!("failed to start server instance {}", i));
        let client = reqwest::Client::new();

        // Create a new term in each session
        let create_url = format!("{}/terms", api(&srv.base_url));
        let body = json!({
            "lemma_en": format!("term_{}", i),
            "lemma_ja": format!("用語_{}", i)
        });

        let resp = client.post(&create_url).json(&body).send().await.unwrap();
        assert_eq!(resp.status().as_u16(), 201);

        let created: serde_json::Value = resp.json().await.unwrap();
        term_ids.push(created["id"].as_str().unwrap().to_string());

        // Verify all previous terms still exist
        for prev_id in &term_ids {
            let get_url = format!("{}/terms/{}", api(&srv.base_url), prev_id);
            let resp = client.get(&get_url).send().await.unwrap();
            assert_eq!(resp.status().as_u16(), 200,
                "Previous term {} should still exist in session {}", prev_id, i);
        }

        srv.stop();
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    }

    // Final verification: all terms should exist
    let srv_final = TestServer::spawn_with_db_path(db_path.clone())
        .await
        .expect("failed to start final server instance");
    let client = reqwest::Client::new();

    let list_url = format!("{}/terms", api(&srv_final.base_url));
    let resp = client.get(&list_url).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    let terms: serde_json::Value = resp.json().await.unwrap();
    let items = terms["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "All 3 terms should persist through multiple restarts");

    srv_final.stop();
}