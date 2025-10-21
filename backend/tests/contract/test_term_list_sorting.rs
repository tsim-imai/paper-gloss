use crate::common::*;
use sqlx::SqlitePool;

async fn create_term(base: &str, en: &str, ja: &str) -> String {
    let url = format!("{}/terms", api(base));
    let body = serde_json::json!({
        "lemma_en": en,
        "lemma_ja": ja
    });
    let resp = reqwest::Client::new().post(&url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201);
    let json: serde_json::Value = resp.json().await.unwrap();
    json["id"].as_str().unwrap().to_string()
}

async fn insert_paper_chunk(pool: &SqlitePool) -> (String, String) {
    let paper_id = uuid::Uuid::new_v4().to_string();
    let chunk_id = uuid::Uuid::new_v4().to_string();
    // Paper
    sqlx::query(
        r#"
        INSERT INTO papers (id, title, source_url, file_path, status, created_at, updated_at)
        VALUES (?1, 'Terms Test Paper', NULL, ?2, 'processing', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&paper_id)
    .bind(format!("artifacts/papers/{}/source.pdf", &paper_id))
    .execute(pool)
    .await
    .unwrap();
    // Chunk
    sqlx::query(
        r#"
        INSERT INTO chunks (id, paper_id, index_, src_text, trans_html, content_hash, token_count, status, retry_count, error_message, created_at, updated_at)
        VALUES (?1, ?2, 0, 'text', '訳文', ?3, NULL, 'translated', 0, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&chunk_id)
    .bind(&paper_id)
    .bind(uuid::Uuid::new_v4().to_string())
    .execute(pool)
    .await
    .unwrap();
    (paper_id, chunk_id)
}

async fn insert_occurrences(pool: &SqlitePool, term_id: &str, paper_id: &str, chunk_id: &str, n: i32) {
    for i in 0..n {
        sqlx::query(
            r#"
            INSERT INTO occurrences (id, term_id, paper_id, chunk_id, start_pos, end_pos, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(term_id)
        .bind(paper_id)
        .bind(chunk_id)
        .bind(i * 10)
        .bind(i * 10 + 5)
        .execute(pool)
        .await
        .unwrap();
    }
}

/// sort=frequency で occurrence_count の降順になること
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t073_terms_sort_by_frequency_desc() {
    let srv = TestServer::spawn().await.expect("server");
    let pool = srv.connect_db().await.expect("db");

    // Create three terms
    let t1 = create_term(&srv.base_url, "alpha", "アルファ").await;
    let t2 = create_term(&srv.base_url, "beta", "ベータ").await;
    let t3 = create_term(&srv.base_url, "gamma", "ガンマ").await;

    // Insert occurrences: t1=3, t2=1, t3=0 (use distinct chunks to avoid UNIQUE constraint)
    let (paper_id, chunk1) = insert_paper_chunk(&pool).await;
    // second chunk in same paper
    let chunk2 = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO chunks (id, paper_id, index_, src_text, trans_html, content_hash, token_count, status, retry_count, error_message, created_at, updated_at)
        VALUES (?1, ?2, 1, 'text2', '訳文2', ?3, NULL, 'translated', 0, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
    )
    .bind(&chunk2)
    .bind(&paper_id)
    .bind(uuid::Uuid::new_v4().to_string())
    .execute(&pool)
    .await
    .unwrap();

    insert_occurrences(&pool, &t1, &paper_id, &chunk1, 3).await;
    insert_occurrences(&pool, &t2, &paper_id, &chunk2, 1).await;

    // Query sort=frequency
    let url = format!("{}/terms?sort=frequency&limit=10", api(&srv.base_url));
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    let items = json["items"].as_array().unwrap();
    let ids: Vec<String> = items.iter().map(|x| x["id"].as_str().unwrap().to_string()).collect();
    assert!(ids.len() >= 3);
    assert!(ids[0] == t1 && ids[1] == t2, "expected order t1(3) > t2(1) > t3(0)");
}

/// sort=recent で updated_at の新しい順になること
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t074_terms_sort_by_recent_desc() {
    let srv = TestServer::spawn().await.expect("server");

    let older = create_term(&srv.base_url, "older", "オルダー").await;
    // ensure different updated_at
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let newer = create_term(&srv.base_url, "newer", "ニューアー").await;

    let url = format!("{}/terms?sort=recent&limit=10", api(&srv.base_url));
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    let items = json["items"].as_array().unwrap();
    let first_id = items[0]["id"].as_str().unwrap();
    assert_eq!(first_id, newer, "most recent term should come first");
    // Also check total_pages math for trivial case
    assert!(json["total"].as_i64().unwrap() >= 2);
}

/// limit 上限100と total/total_pages 計算
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t075_terms_limit_cap_and_totals() {
    let srv = TestServer::spawn().await.expect("server");

    // Create 120 terms
    for i in 0..120 {
        let en = format!("term-{:03}", i);
        let ja = format!("用語-{:03}", i);
        let _ = create_term(&srv.base_url, &en, &ja).await;
    }

    let url = format!("{}/terms?limit=1000", api(&srv.base_url));
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();

    assert_eq!(json["limit"].as_i64().unwrap(), 100, "limit must be capped at 100");
    assert_eq!(json["total"].as_i64().unwrap(), 120);
    assert_eq!(json["total_pages"].as_i64().unwrap(), 2);
    assert!(json["items"].as_array().unwrap().len() <= 100);
}
