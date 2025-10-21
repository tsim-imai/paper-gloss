use crate::common::*;

/// FR-023: 英語のハイフン/アンダースコア/スペースの正規化で同一検索結果になること
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr023_en_hyphen_and_underscore_equivalence() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Create a canonical term: "neural network"
    let create_url = format!("{}/terms", api(&srv.base_url));
    let body = serde_json::json!({
        "lemma_en": "neural network",
        "lemma_ja": "ニューラルネットワーク"
    });
    let resp = client.post(&create_url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201);

    // hyphen form
    let q1 = format!("{}/terms?q={}&lang=en&sort=alphabetical&page=1&limit=50", api(&srv.base_url), "neural-network");
    let res1 = reqwest::get(&q1).await.unwrap();
    assert_eq!(res1.status().as_u16(), 200);
    let json1: serde_json::Value = res1.json().await.unwrap();
    assert!(json1["items"].as_array().unwrap().iter().any(|it| it["lemma_en"] == "neural network"),
            "hyphen form should match the canonical term");

    // underscore form
    let q2 = format!("{}/terms?q={}&lang=en&sort=alphabetical&page=1&limit=50", api(&srv.base_url), "neural_network");
    let res2 = reqwest::get(&q2).await.unwrap();
    assert_eq!(res2.status().as_u16(), 200);
    let json2: serde_json::Value = res2.json().await.unwrap();
    assert!(json2["items"].as_array().unwrap().iter().any(|it| it["lemma_en"] == "neural network"),
            "underscore form should match the canonical term");
}

/// FR-023: 日本語の中点「・」正規化（例: ニューラル・ネットワーク → ニューラルネットワーク）
/// 現状の実装では未対応のため RED になる可能性があります（仕様に沿った期待を先に固めます）。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn fr023_ja_middle_dot_normalization() {
    let srv = TestServer::spawn().await.expect("failed to start test server");
    let client = reqwest::Client::new();

    // Create canonical term without middle dot
    let create_url = format!("{}/terms", api(&srv.base_url));
    let body = serde_json::json!({
        "lemma_en": "neural network",
        "lemma_ja": "ニューラルネットワーク"
    });
    let resp = client.post(&create_url).json(&body).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 201);

    // Query with middle dot variant
    let q = format!("{}/terms?q={}&lang=ja&sort=alphabetical&page=1&limit=50", api(&srv.base_url), "ニューラル・ネットワーク");
    let res = reqwest::get(&q).await.unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let json: serde_json::Value = res.json().await.unwrap();
    assert!(json["items"].as_array().unwrap().iter().any(|it| it["lemma_ja"].as_str().unwrap().contains("ニューラルネットワーク")),
            "middle dot normalization should match canonical Japanese lemma");
}

