# 要件定義（MVP）

更新日: 2025-10-22

## 決定事項（スタック/前提）
- フロントエンド: TypeScript（フレームワークは後決定。Next.js/SvelteKit等を想定）
- バックエンド: Rust（Axum + utoipaでREST/Schema定義）
- データベース: SQLite（ローカル永続化）
- LLM 接続: OpenAI互換エンドポイント `POST /v1/chat/completions` のみ利用
  - 最大トークン: 30,000
  - 並列数: 最大 10 リクエスト
  - API ベースURL例: `AI_API_BASE=http://localhost:8000/v1`、`AI_API_KEY=dummy`
- 数式・図表: MVPは原文PDFを併置（訳文はテキスト/HTML中心）
- 用語解説: 日本語のみ生成・保持（英語解説は不要）
- 学習（SRS）: MVPスコープ外
- 保存場所: 
  - `artifacts/` 動的生成物（原文PDF、抽出テキスト、チャンク、翻訳、抽出結果など）
  - `data/` 静的・再現用（初期辞書や固定設定など）

---

## 目的（プロダクトゴール）
英語論文を日本語で読み進められるようにしつつ、機械学習関連の用語を抽出・辞書化し、訳文中のハイライトとツールチップで素早く理解を支援する。用語集ビューで同時に復習可能とする。

---

## MVP スコープ（機能要件）
1) 論文取り込み
- PDFアップロード、またはURL入力で自動ダウンロード（PDF前提）。
- `artifacts/papers/{paper_id}/source.pdf` に保存。メタ情報（タイトル/URL）をDBへ。

2) テキスト抽出 & チャンク化
- PDFからテキスト抽出（OCRなし）。抽出不可のページは警告のみで継続。
- チャンク化は段落/見出し優先。困難な場合は固定長で分割。
- 目安: 800–1200語/チャンク、10–15%オーバーラップ。
- チャンク内容ハッシュで再処理を回避（キャッシュ）。

3) タグ保持翻訳（LLM）
- 翻訳前に、英語原文の用語を LLM アシストでセンチネルタグ（`[[T:uuid]]…[[/T]]`）でマーキング。
  - 辞書/ヒューリスティックは LLM への補助コンテキストとして提示（優先語・除外語をヒント化）。
  - LLM応答は「タグ付きテキスト」と「タグ一覧（id, surface, lemma_en 等）」のJSONを返却。
- タグを保持したまま `/v1/chat/completions` で日本語に翻訳（LLMには「タグ厳守」を明示）。
- 訳文からタグを解析し、日本語側スパンに対して出現位置（occurrences）を確定。
- 最終HTMLはタグを `<span class="term" data-term-id data-occurrence-id>` に変換。

4) 用語管理（LLMタグ由来）
- STEP3でLLMが付与したタグ情報（lemma_en/surface 等）を基に `terms`/`term_variants` に登録。
- `terms.lemma_ja` は訳文内で初出の日本語表記を暫定正規形として採用（後で編集/マージ可能）。

5) 用語解説（LLM）
- 登録済み用語に対して日本語で2–3文の簡潔解説を生成・更新。
- 重要語から優先キュー処理（頻度やTF-IDF等は将来）。

6) 訳文ビュー＋ツールチップ
- 訳文をページ表示。タグ由来の `<span.term>` をハイライト、ホバー/クリックでツールチップ（用語情報・定義）。
- 右ペインに用語集を並行表示（検索/ソート/手動追加/編集）。
- 原文PDFはタブ/スプリットで併置可能（PDF.js等、MVPは埋め込み程度）。

7) 用語管理（手動操作）
- 両言語で検索・追加・編集・削除。
- 重複候補（表記違い）を提示し、1クリックで統合（マージ）。

8) 並列・再実行と監査
- LLM呼び出しの並列数は環境変数 `AI_MAX_CONCURRENCY` で制御（既定5、上限10想定）。失敗は指数バックオフで自動リトライ。
- タグ保持率のフェイルセーフ: 戻り訳文に含まれるタグの回収率が閾値（既定98%）未満なら自動リトライ（実装予定）。
- 各チャンクのプロンプト/レスポンス（タグ付与後の英文・LLM出力）を保存（実装中の段階で拡充）。

---

## 非機能要件
- ローカル実行前提（個人利用）。外部ネットワークは論文ダウンロード時のみ。
- SQLiteを単一バイナリで同梱（マイグレーションは起動時/CLIで適用）。
- 処理進捗の可視化（取り込み→抽出→翻訳→用語処理）。
- 失敗チャンクはスキップ継続・UIから再試行可能に。
- ログは標準出力＋ファイル（最低: INFO/ERROR）。

---

## データモデル（概略）
- `papers`:
  - `id`, `title`, `source_url`, `file_path`, `status`, `created_at`
- `chunks`:
  - `id`, `paper_id`, `index`, `src_text`, `trans_html`, `token_map_json`
- `terms`（概念の主辞）:
  - `id`, `slug`, `lemma_en`, `lemma_ja`, `reading_kana`, `pos`, `tags`, `note`, `created_at`
- `term_variants`（表記の揺れ）:
  - `id`, `term_id`, `lang`(en/ja), `surface`
- `definitions`（日本語解説）:
  - `id`, `term_id`, `lang`(ja固定), `text`, `provider`, `updated_at`
- `occurrences`（出現位置）:
  - `id`, `term_id`, `paper_id`, `chunk_id`, `start`, `end`, `surface`, `method`, `variant_id?`
    - `method`: 'tagged-translation'（タグ保持翻訳由来）を既定値に追加

> 備考: `terms`は概念中心。英語/日本語/表記揺れは`term_variants`に集約して相互検索可能にする。

---

## 表記揺れポリシー（検索正規化）
- 英語: 小文字化、ハイフン/空白のゆらぎ吸収、素朴な単数化（将来はlemmatization導入）。
- 日本語: 全角/半角統一、カタカナ正規化、中点・長音のゆらぎ吸収。
- 検索は両言語・両表記から同一`term`にヒットすること。

---

## API（下位互換を意識した草案）
- `POST /api/papers/import`（file or url）→ `paper_id`
- `POST /api/papers/{id}/process`（非同期で抽出/翻訳/抽出/解説を順次実行）
- `GET  /api/papers/{id}/translation`（段落/チャンクごとのHTMLを返却）
- `GET  /api/terms?q=&lang=`（両言語検索、正規化照合）
- `POST /api/terms`（手動登録: lemma_en/ja, variants, note）
- `PATCH /api/terms/{id}`（編集/統合）
- `POST /api/terms/{id}/define`（AIによる日本語解説の生成・更新）
- `GET  /api/occurrences?paper_id=`（出現位置の列挙）

変更点（タグ保持翻訳）:
- `/translation` はタグを `<span class="term" data-term-id data-occurrence-id>` に展開済みのHTMLを返す。
- 返却の順序や構造は従来通り（チャンク昇順）。

> OpenAPIはutoipaで自動生成（型定義はRust側に集約）。

---

## LLM プロンプト方針（MVP）
- 事前タグ付け（system の例）:
  - 「あなたは英語科学論文の用語アノテータです。入力テキストから専門用語・固有名詞を抽出し、原文テキストを変更せずにセンチネルタグ `[[T:ID]]` と `[[/T]]` で囲って返してください。タグ以外の文字は一切変更しないでください。出力は JSON で、`tagged_text` と `tags:[{id, surface, lemma_en, pos?}]` を含めてください。タグIDは一意のUUIDです。」
- 事前タグ付け（user の例）:
  - 「テキスト: ...\n 優先語: ...\n 除外語: ...\n 上限: 200件。最長一致・辞書語優先。ネスト禁止。」
- 翻訳（system の例）:
  - 「あなたは科学技術論文の翻訳者です。数式・記号・参照は保持し、平易な日本語に訳します。センチネルタグ `[[T:...]]`/`[[/T]]` は厳密に保持し、タグ内のみ翻訳してください。」
- 翻訳（user の例）:
  - 「次のタグ付きテキストを翻訳してください。タグは保持してください。」+ タグ付け済みチャンク本文
- 解説（user の例）:
  - 「次の用語を日本語で2–3文で簡潔に説明してください。専門外にも伝わる要点重視。」

---

## ディレクトリ構成（予定）
```
artifacts/
  papers/{paper_id}/
    source.pdf
    text/
    translations/
    terms/
data/
  seeds/
  config/
```

---

## 受け入れ基準（MVP）
- 任意のPDF/URLを投入し、訳文がページで閲覧できる。
- 訳文内でタグ由来の `<span.term>` がハイライトされ、ツールチップに日本語解説が表示される。
- 用語集で両言語検索・手動登録・編集・統合ができる。
- LLM呼び出しは `AI_MAX_CONCURRENCY`（既定5）を順守し、失敗チャンクはリトライ可能。

---

## 環境変数（抜粋）
- `AI_API_BASE`: OpenAI互換エンドポイントのベースURL（例: `https://api.example.com/v1`）
- `AI_API_KEY` : 認証トークン
- `AI_MAX_CONCURRENCY`: LLM同時実行の上限（整数、既定5）
- `AI_REQUEST_TIMEOUT_SECS`: LLMリクエストのHTTPタイムアウト秒（既定600=10分）
- タグ保持率が既定閾値（98%）未満の場合は自動でリトライされ、最終的に閾値以上である。

---

## スコープ外（非要件）
- OCR、数式のLaTeX厳密再構成、図表の自動解釈
- 英語解説の生成・保持
- SRS（間隔反復）、Ankiエクスポート
- Embeddings/ベクトル検索、クラスタリング（将来の展望へ）
- ユーザー管理/認可（ローカル前提のため）

---

## 将来の展望
- セクション/全体サマリー生成
- Embeddingsを用いた論文クラスタリング/関連提示
- 用語グラフ（共起/引用リンク）可視化
- チャット検索→自動DL→翻訳のパイプライン化
- 復習（SRS）/クイズ生成/Ankiエクスポート
- OCR、数式・図表の高度処理

---

## 未決事項 / オープンクエスチョン
- フロントフレームワークの確定（Next.js vs SvelteKit 等）
- チャンクサイズの正確な基準（トークン/文字/段落）
- PDF抽出ライブラリ選定（Rust側で済ませるか、外部コマンド連携か）
- タグ生成ヒューリスティックの詳細な重み付け/辞書の更新ポリシー
- 用語の重要度スコアリング（頻度、TF-IDF、ルール）
