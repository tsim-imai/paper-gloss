use crate::common::*;

/// English normalization duplicates ("neural network" vs "neural-network") should be detected
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t076_duplicates_detects_english_normalized_pairs() {
    let srv = TestServer::spawn().await.expect("server");
    let client = reqwest::Client::new();

    // Create two terms that normalize to the same English lemma
    let create_url = format!("{}/terms", api(&srv.base_url));
    // Use double space in first to avoid slug collision while normalizing equal
    let a = client.post(&create_url).json(&serde_json::json!({
        "lemma_en": "neural  network",
        "lemma_ja": "ニューラルネットワーク"
    })).send().await.unwrap().json::<serde_json::Value>().await.unwrap();
    let b = client.post(&create_url).json(&serde_json::json!({
        "lemma_en": "neural-network",
        "lemma_ja": "ニューラル・ネットワーク"
    })).send().await.unwrap().json::<serde_json::Value>().await.unwrap();

    // Call duplicates API
    let url = format!("{}/terms/duplicates", api(&srv.base_url));
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();

    let dups = json["duplicates"].as_array().unwrap();
    assert!(json["total"].as_u64().unwrap() >= 1);

    // Ensure a pair with our two IDs exists (order-agnostic)
    let id_a = a["id"].as_str().unwrap();
    let id_b = b["id"].as_str().unwrap();
    let found = dups.iter().any(|p| {
        let t1 = p["term1"]["id"].as_str().unwrap();
        let t2 = p["term2"]["id"].as_str().unwrap();
        (t1 == id_a && t2 == id_b) || (t1 == id_b && t2 == id_a)
    });
    assert!(found, "expected duplicate pair for english-normalized lemmas");
}

/// Japanese normalization duplicates (middle dot variants) should be detected
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn t076_duplicates_detects_japanese_normalized_pairs() {
    let srv = TestServer::spawn().await.expect("server");
    let client = reqwest::Client::new();

    // Create two terms that normalize to same Japanese lemma
    let create_url = format!("{}/terms", api(&srv.base_url));
    // Use double space variant so both normalize to the same form
    let a = client.post(&create_url).json(&serde_json::json!({
        "lemma_en": "something a",
        "lemma_ja": "ディープ  ラーニング"
    })).send().await.unwrap().json::<serde_json::Value>().await.unwrap();
    let b = client.post(&create_url).json(&serde_json::json!({
        "lemma_en": "something b",
        "lemma_ja": "ディープ・ラーニング"
    })).send().await.unwrap().json::<serde_json::Value>().await.unwrap();

    let url = format!("{}/terms/duplicates", api(&srv.base_url));
    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    let dups = json["duplicates"].as_array().unwrap();

    let id_a = a["id"].as_str().unwrap();
    let id_b = b["id"].as_str().unwrap();
    let found = dups.iter().any(|p| {
        let t1 = p["term1"]["id"].as_str().unwrap();
        let t2 = p["term2"]["id"].as_str().unwrap();
        (t1 == id_a && t2 == id_b) || (t1 == id_b && t2 == id_a)
    });
    assert!(found, "expected duplicate pair for japanese-normalized lemmas");
}
