# タグ保持翻訳 仕様（Tagged Translation Spec）

更新日: 2025-10-22

本仕様は Import → Tagging → Tag保持翻訳 → タグ解析 → occurrences作成 → 定義生成 の新フローを定義します。既存DBは破棄済みであり、今後のインポートから適用します。

---

## 1. 用語タグの基本

- 記法（候補A採用）
  - 開始: `[[T:{ID}]]`  終了: `[[/T]]`
  - `{ID}` は `uuid-v4`（小文字、ハイフン含む）
  - ネスト不可、重複ID不可、未クローズ禁止
  - タグと本文の間に余計な空白を入れない（LLMが紛れにくい）

- 例
  - 入力英語: `We propose the [[T:3f8c...]]Gaussian Process[[/T]] model.`
  - 出力日本語（LLM）: `我々は [[T:3f8c...]]ガウス過程[[/T]] モデルを提案する。`
  - レンダリング: `<span class="term" data-term-id="TERM_ID" data-occurrence-id="OCC_ID">ガウス過程</span>`

---

## 2. パイプライン

1) Import
- PDF保存 → `papers` 作成 → 非同期処理開始。

2) Tagging（英語原文・LLMアシスト）
- 入力: 英語チャンク `src_text`、補助コンテキスト（優先語/除外語/上限）
- LLM出力（JSON）:
  ```json
  {
    "tagged_text": "...[[T:uuid]]surface[[/T]]...",
    "tags": [
      { "id": "uuid", "surface": "surface as in text", "lemma_en": "Gaussian process", "pos": "noun" }
    ]
  }
  ```
- バリデーション:
  - `strip_tags(tagged_text)` が原文と完全一致か
  - タグの対応（開/閉）とID一意性、ネストなし
  - タグ数≦200、`surface` が原文に厳密一致
  - 合格しない場合は自動リトライ（最大3回）。以降も不合格なら辞書ベースにフォールバック可能（任意）。
- TagMap生成: `tagged_text` からタグ位置を機械的に復元し `{ tag_id, paper_id, chunk_index, start, end, surface, lemma_en }` を作成

3) Tag保持翻訳（LLM）
- 入力: `tagged_en`
- System指示: 「タグ `[[T:...]]`/`[[/T]]` は厳密保持、タグ内のみ翻訳、順序・対応を壊さない」
- 並列: 最大10リクエスト、指数バックオフリトライ
- 監査: 入出力を保管（PII無し）

4) タグ解析 → JAスパン確定
- 入力: LLM出力 `tagged_ja`
- 出力: 日本語スパン `start/end`、表記 `surface_ja`
- 手順: `tagged_ja` からタグ対を抽出 → タグ除去時のオフセット再計算 → `occurrences` を作成
  - `occurrences.method='tagged-translation'`
  - `occurrences.surface=surface_ja` を保存
  - `occurrences.variant_id` が既存にない場合は `NULL`

5) 用語登録（LLMタグ由来）
- `terms`/`term_variants` 登録
  - `lemma_en`: 既存突合→なければ新規
  - `lemma_ja`: 訳文で初出の `surface_ja` を暫定正規形として採用
  - `term_variants`: `lang='en'|'ja'` で出現表記を順次追加（重複はUNIQUEで保護）

6) 定義生成（AI）
- `occurrences` に出現した `term_id` を対象に日本語定義をLLMで生成・更新

---

## 3. 日本語表記揺れの扱い

- 正規化（保存・比較ともに適用）
  - NFKC、全角/半角統一
  - 長音「ー」、中点「・」、空白のゆらぎ吸収
  - カタカナ⇄ひらがな変換（比較時のみ）
  - 小書き促音・拗音の正規化
- マッチングはタグ起点のため原則不要だが、`term_variants(ja)` 拡充時に上記ルールで近似同定する

---

## 4. データモデル拡張

### occurrences（追加）
- `surface` TEXT            … 訳文で実際にハイライトした文字列
- `method`  TEXT NOT NULL   … 'tagged-translation' を既定
- `variant_id` TEXT NULL    … `term_variants.id` に紐付け（あれば）

制約/インデックス（推奨）
- CHECK(method IN ('tagged-translation'))
- INDEX idx_occurrences_method(method)

### terms/term_variants
- `terms.lemma_ja` は初出の `surface_ja` を暫定正規形として保存（後で手動編集/マージ可）

---

## 5. API 影響

- `GET /api/papers/{id}/translation`
  - 返却する `translated_text` はタグを `<span class="term" data-term-id data-occurrence-id>` に展開済み
  - 並び順・ペイロード形状は従来通り

- `GET /api/occurrences?paper_id=`
  - `surface/method/variant_id` を含む

---

## 6. フェイルセーフと品質

- タグ保持率: `回収タグ数 / 送信タグ数 >= 0.98` を合格
- 未知タグ/改変タグは無視、既知タグのみ採用
- 連続リトライ上限: 3回（指数バックオフ）
- 失敗チャンクは `chunks.status='failed'`、UIから再試行可
 - 事前タグ付けも同様に、整合チェック不合格時は自動リトライ。

---

## 7. 制限と性能

- チャンク長: 800–1200語、オーバーラップ10–15%
- タグ上限: 200/チャンク（辞書語を優先）
- 並列数: `AI_MAX_CONCURRENCY` で制御（既定5、上限10想定）。事前タグ付け・翻訳いずれも同値。
- LLM HTTPタイムアウト: `AI_REQUEST_TIMEOUT_SECS`（既定600=10分）

---

## 8. ロギング/監査

- 保存対象
  - `tagged_en`（送信前）、`tagged_ja`（受信後）
  - TagMap（tag_id→位置/用語）
  - LLM呼出しメタ（遅延、リトライ回数）

---

## 9. 互換と移行

- 新フローは今後のインポートに適用（既存DBは破棄済み）
- 旧「LLMによる用語抽出」ステップはMVPから除外（必要なら将来オプションに再導入）
