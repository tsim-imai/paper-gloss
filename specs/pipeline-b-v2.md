# Pipeline B v2 — 日本語用語抽出（LLM）仕様

更新日: 2025-10-23

スコープ
- A（翻訳）済みの日本語訳から、辞書エントリを抽出・登録する。
- POS は廃止。`variants=表記ゆれ` と `aliases=同義/略語/別名` を分離保存。
- 抽出はベストエフォート。0件は `completed_empty`。

前提
- `papers/{id}` が存在し、該当 paper の `chunks.trans_html` が利用可能。

エンドポイント
- `POST /api/papers/{id}/extract-terms-jp`
  - 入力: `{ "max_terms"?: number, "min_confidence"?: number }`（任意。既定 `min_confidence=0.0`）
  - 出力: `{ result_state, added_terms, added_variants, added_aliases }`
  - 並列: `AI_MAX_CONCURRENCY`（チャンク単位 or セクション単位で分割）

LLM 出力（中間 JSON 仕様）
```json
{
  "schema_version": "b2.0",
  "terms": [
    {
      "lemma_ja": "畳み込みニューラルネットワーク",
      "lemma_en": "convolutional neural network",
      "reading_kana": "タタミコミ ニューラル ネットワーク",
      "confidence": 0.83,
      "variants": {
        "ja": ["畳み込みNN", "畳み込みネット"],
        "en": ["CNN", "convnet"]
      },
      "aliases": [
        { "surface": "CNN", "lang": "en", "kind": "abbrev", "confidence": 0.9 },
        { "surface": "コンブネット", "lang": "ja", "kind": "alias", "confidence": 0.5 }
      ]
    }
  ]
}
```

登録ルール
- term（概念）の代表表記: 原則 `lemma_ja`（存在しない場合は `lemma_en`）。
- `slug` 生成: `lemma_en` があれば kebab-case（英数のみ）。無ければ `lemma_ja` をローマ字転写→kebab。
- duplicates: `terms.slug` または（`lemma_ja`/正規化, `lemma_en`/正規化）のいずれか一致で同一。
- `term_variants`: 表記ゆれのみ（言語別）。
- `term_aliases`: 同義・略語・別名。variants と重複する surface は aliases を優先（意味表現として保持）。
- confidence: 0–1 を保存（NULL 可）。

正規化/重複除去
- 英語: 小文字化、ハイフン/スペースの統一、素朴単数化。
- 日本語: 全半角/カタカナ/長音/中点の揺れ吸収。
- `term_variants` と `term_aliases` は（term_id, lang, surface[, kind]) で一意化。

Idempotency & 再実行
- 同一 paper で再実行可能。既存一致はスキップ、初出のみ追加。
- `added_terms/variants/aliases` は今回追加分のみをカウント。
- 既存 term の `lemma_ja/lemma_en` は自動更新しない（手動編集のみ）。

失敗/結果状態
- `completed_nonempty` | `completed_empty` | `failed`
- 409/412 は前提未満足（翻訳未完了など）、5xx は LLM/システム障害。

プロンプト指針（要点）
- 出力は JSON のみ。本文は要約しない。
- 一般語は除外（学術用語/固有名中心）。
- 同義・略語は `aliases` に、スペル違い/短縮表記は `variants` に。

受け入れ条件（抜粋）
- POS を出力せず登録しない。
- 同一 surface が variants/aliases の両方に現れる場合、aliases を優先保存。
- スキーマ v2 に従い DB に保存（`term_aliases`, `term_tags` は B では更新しない）。
