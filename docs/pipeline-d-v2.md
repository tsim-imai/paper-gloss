# Pipeline D v2 — 用語解説/タグ/関係の拡張設計

更新日: 2025-10-23

本書は Pipeline D（Generate Definitions）の拡張版設計（v2）を示す。MVPの「2–3文の日本語解説」から、学習/復習に役立つ構造化情報へ拡張する。既存の API/DB を尊重しつつ、段階的に導入可能な最小セットを定義する。

---

## 目標
- 定義の「質と使い勝手」を底上げ（短い要約＋用例＋関連語）。
- 用語間の連関（is-a / uses / measured-by など）を保存し、横断ナビゲーションと検索精度を向上。
- タグ体系で「何の用語か」を素早く把握（Task/Model/Dataset/Metric…）。
- 自動生成の品質を可視化（信頼度/レビュー状態/再生成方針）。

---

## 出力スキーマ（LLM→アプリ間の中間JSON）

LLM からの生成物は下記 JSON を推奨とする（将来の拡張互換のため `schema_version` を付与）。

```json
{
  "schema_version": "d2.0",
  "term_id": "uuid",
  "lang": "ja",
  "generation": {
    "summary": "2-3文の簡潔な定義",
    "long": "必要なら詳しめの説明（任意）",
    "usage_examples": ["論文文脈に即した1-3例文"],
    "confusables": ["似た概念や誤用されやすい語"],
    "see_also": ["関連して調べると良い語"],
    "quality": {"confidence": 0.78, "flags": ["needs_review?", "uncertain_math?"]},
    "meta": {"provider": "ai", "model": "…", "prompt_version": "d2.0"}
  },
  "tags": {
    "term_type": ["Task", "Model", "Dataset", "Metric", "Algorithm", "Math"],
    "domains": ["CV", "NLP", "Speech", "RL", "Graph", "TimeSeries", "Multimodal"],
    "data_types": ["Image", "Text", "Audio", "Video", "Tabular"],
    "level": "Beginner|Intermediate|Advanced"
  },
  "aliases": [
    {"surface": "略語", "kind": "abbrev", "lang": "en|ja"},
    {"surface": "別名/同義", "kind": "synonym", "lang": "en|ja"}
  ],
  "relations": [
    {"to_term_slug_or_id": "…", "type": "is-a|part-of|uses|trained-on|measured-by|compares-to|improves-upon", "confidence": 0.72, "evidence": "抜粋文"}
  ]
}
```

保存は下記の DB 追加で段階導入する（SQLite 前提）。

---

## DB拡張（最小セット）

1) 定義メタ/品質
- `definition_meta`: `id`, `term_id`, `provider`, `model`, `prompt_version`, `confidence`(REAL), `flags`(TEXT JSON), `updated_at`
  - 備考: 既存 `definitions` の1:1拡張。上書き更新時に世代管理が必要なら `definition_revisions` を導入（将来）。

2) タグ
- `term_tags`: `term_id`, `tag_group`(term_type/domains/data_types/level/custom), `tag_value`, `confidence`(REAL), `source`(ai|human), `created_at`
  - 備考: 文字列タグを正規化する `tags_master` は必要になってからでよい。

3) 用語間関係
- `term_relations`: `id`, `src_term_id`, `dst_term_id`, `relation_type`, `confidence`(REAL), `evidence`(TEXT), `paper_id?`, `created_at`
  - 代表的 `relation_type`: `is-a`, `part-of`, `uses`, `trained-on`, `measured-by`, `compares-to`, `improves-upon`。

4) エイリアス（同義/略語）
- 既存 `term_variants` は表記ゆれ（orthographic）中心。意味上の別名は `term_aliases` を新設：
  - `term_aliases`: `id`, `term_id`, `surface`, `lang`, `kind`(synonym|abbrev|alias), `confidence`(REAL), `created_at`

5) 埋め込み（任意・後付け）
- `term_embeddings`: `id`, `term_id`, `lang`, `model`, `vector`(BLOB)
  - 備考: まずはアプリ側計算+フィルタで十分。拡張で類似語推薦や関連語の自動補完に活用。

---

## D v2 の処理段階（段階導入可）

- D0: コンテキスト収集（機械）
  - Cで得た `occurrences` から代表的な出現文を3–5件抽出。周辺文と共に「用例パック」を作成。
- D1: 定義生成（LLM）
  - `summary`（必須）と `usage_examples`（任意）に限定してまず導入。出力は上記 JSON で受け取り、既存 `definitions.text` は `summary` を格納。
- D2: タグ付け（LLM）
  - `tags.term_type/domains` を優先生成（少数精鋭の選択肢方式）。
- D3: 関係抽出（LLM）
  - is-a / uses / measured-by / trained-on を優先。エビデンスとして該当文の抜粋を保存。
- D4: エイリアス生成（LLM）
  - 略語・同義語を追加。既存 `term_variants` とは役割を分離。
- D5: 自己検証（LLM）
  - 1行サマリの再説明（round-trip）と、曖昧性フラグの自己申告を促す（`quality.flags`）。

---

## プロンプト指針（抜粋）

共通 system:

```
あなたは機械学習分野の日本語テクニカルライターです。出力は必ず指定のJSONスキーマに従います。事実に自信がない場合は不確実フラグを付け、臆測は避けます。定義は初学者にも伝わる簡潔さを優先します。
```

定義生成 user（D1）:

```
対象用語: {lemma_ja} / 英語: {lemma_en}
用例パック（代表出現文）: ...
出力: summary(2-3文), usage_examples(1-3), confusables(0-3), see_also(0-5)
必ず JSON で。
```

タグ付け user（D2）:

```
候補: term_type=[Task, Model, Dataset, Metric, Algorithm, Math, Library, Tool],
domains=[CV, NLP, Speech, RL, Graph, TimeSeries, Multimodal]
制約: 最大 term_type 2つ, domains 2つ, 信頼度0-1を付与。JSONのみ。
```

関係抽出 user（D3）:

```
許可関係: is-a, uses, trained-on, measured-by, improves-upon, compares-to
各関係は evidence に論文の該当文を短く添える。JSONのみ。
```

---

## 評価（自動/人手）

- 自動
  - 形式: JSON検証（必須）/ 空フィールド率 / 長さ制約。
  - 矛盾: summary と long の整合チェック（NLI/LLMジャッジ）。
  - 再現性: 同一入力での温度0再生成一致率（drift検知）。
  - 網羅: 対象論文内トップ頻出語に対する定義充足率。
- 人手
  - 5段階（正確さ・明瞭さ・有用さ）+ 承認/要修正。
  - UI: 差分比較（旧/新）、ワンクリック採択、タグ編集。

---

## POS/固有表現の扱い

- POS（品詞）
  - 用語は名詞優勢のため `terms.pos` は必須ではない。保持するなら粗いカテゴリ（Noun/Verb/Adj/Other）に縮退し、厳密な形態情報は `term_variants` または将来の `variant_features` に寄せる。
- 固有表現（NE）
  - データセット/モデル/指標/手法名などの NE は後活用が大きい。`term_tags.term_type` で区別できるようにし、略語展開（`term_aliases.kind=abbrev`）を併置すると利便性が高い。

---

## 互換性とロールアウト

1. まず D1（summary+usage_examples）だけを導入。UI ツールチップには `summary` のみ表示、拡張表示で用例を折りたたみ表示。
2. 次に D2（タグ）を追加し、用語集にチップ表示とフィルタを実装。
3. D3（関係）を導入し、"関連語" セクションと簡易グラフビューを追加。
4. D4（エイリアス）で検索想起率を改善。
5. D5（自己検証）で品質メタを保存し、レビュー優先度を自動付与。

---

## 追加API（案）

- `POST /api/terms/{id}/define?v=2` — D v2 生成。`summary_only=true` で段階導入。
- `POST /api/terms/{id}/tags` — タグ更新（AI/Human 両対応）。
- `POST /api/terms/{id}/relations` — 関係を追加（バルク）
- `GET  /api/terms/{id}/graph` — 近傍用語と関係の取得。
- `GET  /api/papers/{id}/key-terms` — 頻度/中心性ベースの優先用語列挙。

---

## 開発メモ

- Rust(Axum) 側: まずは `definitions` 既存保存のまま、中間JSONを `artifacts/papers/{id}/definitions.d2.jsonl` として併置保存→後続で DB に正規化移行。
- 並列制御: D を用語単位でバッチに分割（10語/バッチ目安）。`AI_MAX_CONCURRENCY` はバッチ単位で適用。
- テレメトリ: `prompt_version` と `result_state` を Status API へ追加（比較評価用）。

