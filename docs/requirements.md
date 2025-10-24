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

3) パイプライン分離（独立API）
// 旧方針（タグ保持翻訳）は廃止。A/B/C/D に分離。

- Pipeline A: 翻訳（LLM）
  - 責務: チャンクの翻訳を完了させる。
  - 出力: `chunks.trans_html` を保存。
  - paper.status: 全訳了で completed、一部失敗は processing 維持。
  - 失敗ポリシー: 進捗ゼロか致命的例外のみ failed。

- Pipeline B: 日本語用語抽出 + 辞書登録（LLM）
  - 責務: 日本語訳から辞書を拡充（登録/正規化/重複排除）。
  - 出力: 追加/更新件数を返す（0件でも completed_empty）。
  - 失敗ポリシー: 前提未満足（翻訳未完了）や LLM/JSON致命エラーのみ failed。0件はエラー扱いにしない。

- Pipeline C: 日本語機械スキャン（機械）
  - 責務: 辞書と訳文を同期し occurrences を保存。
  - 出力: 作成件数を返す（0件でも completed_empty）。
  - 失敗ポリシー: 前提未満足（翻訳未完了）やDB障害のみ failed。

- Pipeline D: 定義生成（LLM）
  - 責務: 登録済み用語の日本語解説（2–3文）を生成・保存（既存は保持）。
  - 出力: 生成件数を返すほか、result_state（completed_nonempty|completed_empty|failed）。
  - 失敗ポリシー: LLM致命エラーのみ failed。0件はエラーにしない。

4) 用語管理（辞書登録）
- Pipeline B の抽出結果（lemma_ja/lemma_en 等）を基に `terms`/`term_variants`/`term_aliases` に登録（POSは廃止）。
- `terms.lemma_ja` は訳文内で初出の日本語表記を暫定正規形として採用（後で編集/マージ可能）。

5) 用語解説（LLM）
- 登録済み用語に対して日本語で2–3文の簡潔解説を生成・更新。
- 重要語から優先キュー処理（頻度やTF-IDF等は将来）。

6) 訳文ビュー＋ツールチップ
- 訳文をページ表示（プレーン）。JPスキャン後にハイライト、ホバー/クリックでツールチップ（用語情報・定義）。
- 右ペインに用語集を並行表示（検索/ソート/手動追加/編集）。
- 原文PDFはタブ/スプリットで併置可能（PDF.js等、MVPは埋め込み程度）。

7) 用語管理（手動操作）
- 両言語で検索・追加・編集・削除。
- 重複候補（表記違い）を提示し、1クリックで統合（マージ）。

8) 並列・再実行と監査
- LLM呼び出しの並列数は `AI_MAX_CONCURRENCY`（既定5）で制御。失敗は指数バックオフで自動リトライ。
- 各工程は独立APIで再実行可能。0件は completed_empty として扱う（後続で再実行可）。
- ログは翻訳/抽出のプロンプト・レスポンスを保存（PII配慮）。

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
    - `method`: 'jp-scan'（日本語機械スキャン由来）

> 備考: `terms`は概念中心。英語/日本語/表記揺れは`term_variants`に集約して相互検索可能にする。

---

## 表記揺れポリシー（検索正規化）
- 英語: 小文字化、ハイフン/空白のゆらぎ吸収、素朴な単数化（将来はlemmatization導入）。
- 日本語: 全角/半角統一、カタカナ正規化、中点・長音のゆらぎ吸収。
- 検索は両言語・両表記から同一`term`にヒットすること。

---

## API（独立パイプライン）
 - `POST /api/papers/import`（file or url）→ `paper_id`（自動処理なし、status=pending）
 - `POST /api/papers/{id}/translate`（A）
 - `POST /api/papers/{id}/extract-terms-jp`（B）
 - `POST /api/papers/{id}/scan-jp`（C）
 - `POST /api/papers/{id}/generate-definitions`（D）
 - `POST /api/terms/{id}/define`（D: 単語単位の再生成）
 - `GET  /api/papers/{id}/status`（A/B/C/Dの進捗・result_state を返却）
 - `GET  /api/papers/{id}/translation`（チャンクごとのHTML）
 - `GET  /api/terms?q=&lang=`（両言語検索、正規化照合）
 - `POST /api/terms`（手動登録: lemma_en/ja, variants, note）
 - `PATCH /api/terms/{id}`（編集/統合）
 - `GET  /api/occurrences?paper_id=`（出現位置の列挙）

変更点（タグ保持翻訳 → JP-first）:
- `/translation` はタグを `<span class="term" data-term-id data-occurrence-id>` に展開済みのHTMLを返す。
- 返却の順序や構造は従来通り（チャンク昇順）。

> OpenAPIはutoipaで自動生成（型定義はRust側に集約）。

---

## LLM プロンプト方針（MVP）
- 翻訳（system の例）:
  - 「あなたは科学技術論文の翻訳者です。数式・記号・参照は保持し、平易で一貫した日本語に訳します。」
- 翻訳（user の例）:
  - 「次のテキストを日本語に翻訳してください。段落構造は維持してください。」
- JP用語抽出（system の例）:
  - 「あなたは日本語論文の用語アノテータです。出力は JSON 配列のみ。各要素は {lemma_ja, lemma_en, reading_kana?, variants?, aliases?}。本文を変更・要約しない。スパンは返さない。POS は出力しない。」
- JP用語抽出（user の例）:
  - 「テキスト: ...\n 一般語は除外。重複は統合して代表表記を lemma_ja とする。」

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
- 訳文内で JPスキャンに基づくハイライトが表示され、ツールチップに日本語解説が表示される。
- 用語集で両言語検索・手動登録・編集・統合ができる。
- LLM呼び出しは `AI_MAX_CONCURRENCY`（既定5）を順守し、失敗チャンクはリトライ可能。

---

## 環境変数（抜粋）
- `AI_API_BASE`: OpenAI互換エンドポイントのベースURL（例: `https://api.example.com/v1`）
- `AI_API_KEY` : 認証トークン
- `AI_MAX_CONCURRENCY`: LLM同時実行の上限（整数、既定5）
- `AI_REQUEST_TIMEOUT_SECS`: LLMリクエストのHTTPタイムアウト秒（既定600=10分）


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
