# API v2 — B/D まわりのエンドポイント仕様

更新日: 2025-10-23

対象
- Pipeline B v2（日本語用語抽出）
- Pipeline D v2（定義生成 拡張）

共通
- すべて JSON。成功時 200 or 202。前提未満足 409/412。致命 5xx。
- Status 拡張: `definitions.prompt_version`（d1|d2）を返す。

1) B: 日本語用語抽出
- `POST /api/papers/{id}/extract-terms-jp`
  - req: `{ "max_terms"?: number, "min_confidence"?: number }`（既定: `min_confidence=0.0`）
  - res: `{ "result_state": "completed_nonempty|completed_empty|failed", "added_terms": number, "added_variants": number, "added_aliases": number }`
  - side effects: `terms`, `term_variants`, `term_aliases` を作成/更新。

2) D: 定義生成（v2 のみ）
- `POST /api/papers/{id}/generate-definitions`
  - req: `{}`（パラメータ不要、v2固定）
  - res: `{ "result_state": "completed_nonempty|completed_empty|failed", "generated": number, "failed": number, "prompt_version": "d2" }`
  - side effects: `definitions.text`（summary）更新＋中間 JSON を `artifacts/papers/{id}/definitions.d2.jsonl` に追記。`definition_meta` を保存。

- `POST /api/terms/{id}/define`
  - req: `{}`（パラメータ不要、v2固定）
  - res: `{ "ok": true, "prompt_version": "d2" }`

3) Status
- `GET /api/papers/{id}/status`
- 返却拡張例:
  ```json
  {
    "definitions": {
      "generated": 57,
      "failed": 3,
      "last_run_at": "...",
      "status": "completed|processing|idle",
      "result_state": "completed_nonempty|completed_empty|failed",
      "prompt_version": "d2"
    }
  }
  ```
  - 備考: `generated`/`failed` は「直近のD実行で生成/失敗した件数」（今回生成数）を返す。
