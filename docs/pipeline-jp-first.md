# JP-first Pipelines — Split Execution Design

更新日: 2025-10-22

本書は、翻訳/JP用語抽出/JP機械スキャンを独立APIとして分離し、任意のタイミング・回数で実行できるようにする設計を示す。

---

## Pipelines

- A: Translate (LLM)
  - POST /api/papers/{id}/translate — 英→日の翻訳のみを実行。
  - 並列: `AI_MAX_CONCURRENCY`。HTTPタイムアウト: `AI_REQUEST_TIMEOUT_SECS`。
  - 責務: チャンクの翻訳を完了させ、日本語テキストを保存する。
  - 成功条件:
    - completed: 全チャンクが翻訳済み
    - partial: 一部失敗（paper.status は processing 維持）
  - 失敗条件: 入力不正/抽出ゼロ/システム障害で全く進捗がない場合のみ failed（部分成功が1つでもあれば partial 扱い）

- B: Extract Terms (JP, LLM)
  - POST /api/papers/{id}/extract-terms-jp — 日本語訳から {lemma_ja, lemma_en, reading_kana?, variants?, aliases?} を抽出し、辞書へ即登録（POSは廃止）。
  - 責務: 日本語訳から辞書を拡充する（登録/重複排除/正規化）。
  - 成功条件:
    - completed_nonempty: 登録件数 > 0
    - completed_empty: 0件（エラーにしない）
  - 失敗条件: 入力未準備（翻訳未完了）や LLM/JSONパース致命エラー等。HTTPは 409/412（未満足前提）や 502/503 を返す。
  - 重複除去: 完全一致＋正規化一致。

- C: Scan JP (Machine)
  - POST /api/papers/{id}/scan-jp — `term_variants(lang='ja')` を用い、日本語訳にマッチして `occurrences(method='jp-scan')` を作成。
  - 責務: 訳文と辞書を同期し、出現位置を保存する。
  - 成功条件:
    - completed_nonempty: 作成件数 > 0
    - completed_empty: 0件（エラーにしない）
  - 失敗条件: 入力未準備（翻訳未完了）などの前提未満足、DB障害等。

- D: Generate Definitions (LLM, v2 固定)
  - POST /api/papers/{id}/generate-definitions — 当該論文に関連する用語の定義をJSONスキーマで生成し、summary を保存。中間JSONは artifacts へ追記。
  - POST /api/terms/{id}/define — 単語単位で定義を再生成/更新（v2）。
  - 責務: `definitions(lang='ja').text` に summary（2–3文）を保存し、`definition_meta` に品質/由来メタを保存。paper.status へ影響しない。
  - 出力: summary を `definitions.text` に保存。中間JSON（`docs/pipeline-d-v2.md` 参照）を `artifacts/papers/{id}/definitions.d2.jsonl` に追記。
  - 並列: `AI_MAX_CONCURRENCY`（既定5）。HTTPタイムアウト: `AI_REQUEST_TIMEOUT_SECS`（既定600秒）。
  - 成功条件:
    - completed_nonempty: 生成件数 > 0
    - completed_empty: 0件（エラーにしない。C未実行/用語なし等）
  - 失敗条件: LLM致命エラーや前提未満足（論文・用語が存在しない等）。

---

## Status Model

GET /api/papers/{id}/status は以下の小節を返す（例）。

```
{
  "paper_id": "...",
  "status": "processing|completed|failed",
  "translation": { "total_chunks": 42, "completed_chunks": 40, "failed_chunks": 2, "status": "processing" },
  "terms_jp":    { "total_terms": 128, "last_run_at": "...", "status": "completed|processing|idle" },
  "scan_jp":     { "total_occurrences": 532, "last_run_at": "...", "status": "completed|processing|idle" },
  "definitions": { "generated": 57, "failed": 3, "last_run_at": "...", "status": "completed|processing|idle", "result_state": "completed_nonempty|completed_empty|failed", "prompt_version": "d2" }
}
```

`paper.status` は翻訳の進捗で決定（翻訳完了で completed）。B/C は paper.status に影響しないが、各自の `status` と `result_state`（completed_nonempty/completed_empty/failed）を持つ。

各パイプラインの状態遷移:
- idle → running → completed_nonempty|completed_empty|failed

HTTPの推奨:
- 起動系: 202 Accepted + operation id
- 前提未満足: 409 Conflict または 412 Precondition Failed
- 同期完了（小規模）: 200 OK + result_state

---

## Idempotency & Re-run

- /translate: 再実行可（失敗チャンクのみ再試行などの将来最適化は任意）。
- /extract-terms-jp: 再実行で重複除去後に不足分を追加登録。既存と衝突時はスキップ。
- /scan-jp: 再実行時は当該paperの occurrences(method='jp-scan') を一度クリアし、最新辞書で再構築（推奨）。
- /generate-definitions: 既存定義は保持し、未定義の用語のみ対象（上書きモードは将来オプション）。

---

## Failure Policy

- A翻訳が成功すれば論文は完成。B/C が0件でもエラーにしない。
- LLM/ネットワーク障害は指数バックオフで自動リトライ。上限超えは部分失敗のまま完了可。
- D定義生成はベストエフォート。0件でもエラーにしない（completed_empty）。

---

## Prompts (Outline)

- 翻訳: 数式/記号/参照の保持、表現一貫性。
- JP用語抽出: JSON配列のみ、{lemma_ja, lemma_en, reading_kana?, variants?, aliases?}。スパンなし、POSなし。
- 定義生成（v2）: summary に加え、用例/関連語/タグ/関係/品質メタを JSON（`pipeline-d-v2.md`）で返す。summary はDB保存、JSONは artifacts 併置→後続で正規化。

---

## Migration Notes

- 旧「タグ保持翻訳」仕様は廃止。`occurrences.method` は 'jp-scan' を使用。
- Importは自動実行禁止。`pending` で停止し、A/B/Cは明示APIで起動。
 - Dは任意タイミングで実行可能だが、Cの後に実行すると対象用語の同定が容易（occurrences 経由）。
