# Schema v2 — Terms/Definitions 拡張スキーマ

更新日: 2025-10-23

目的
- Pipeline B（用語抽出）と D（定義生成）の拡張に合わせ、辞書スキーマを再設計。
- 後方互換は不要。ただし A（翻訳）由来の `papers/chunks` は保持する。

非対象（変更なし）
- `papers`, `chunks`, `occurrences`（Cは後回し）。

変更点（確定）
- `terms.pos` を削除。
- `terms.tags` を廃止し、正規化テーブル `term_tags` へ移行。
- `term_variants` は「表記揺れ（orthographic variants）」専用に限定。
- 「同義/略語/別名」は新設 `term_aliases` に保存し、variants と分離。
- 定義の品質/由来メタは新設 `definition_meta` へ。

追加テーブル（新設）
- `term_tags`:
  - `id` PK
  - `term_id` FK(terms)
  - `tag_group` TEXT 例: term_type|domains|data_types|level|custom
  - `tag_value` TEXT 例: Model, NLP, Image
  - `confidence` REAL NULL可（0–1）
  - `source` TEXT 例: ai|human
  - `created_at` DATETIME

- `term_aliases`:
  - `id` PK
  - `term_id` FK(terms)
  - `surface` TEXT
  - `lang` TEXT en|ja
  - `kind` TEXT synonym|abbrev|alias
  - `confidence` REAL NULL可
  - `created_at` DATETIME

- `definition_meta`:
  - `id` PK
  - `term_id` FK(terms)
  - `provider` TEXT 例: ai|human
  - `model` TEXT
  - `prompt_version` TEXT 例: d1|d2
  - `confidence` REAL NULL可
  - `flags` TEXT(JSON)
  - `updated_at` DATETIME

将来追加（任意）
- `term_relations`（Dの関係出力が有効化されたら導入）:
  - `id` PK
  - `src_term_id` FK(terms)
  - `dst_term_id` FK(terms)
  - `relation_type` TEXT is-a|part-of|uses|trained-on|measured-by|compares-to|improves-upon
  - `confidence` REAL NULL可
  - `evidence` TEXT
  - `paper_id` FK(papers) NULL可
  - `created_at` DATETIME

既存テーブル（変更後定義）
- `terms`:
  - `id` PK
  - `slug` TEXT UNIQUE
  - `lemma_en` TEXT NULL可
  - `lemma_ja` TEXT NULL可
  - `reading_kana` TEXT NULL可
  - `note` TEXT NULL可
  - `created_at` DATETIME
  - `updated_at` DATETIME

- `term_variants`:
  - `id` PK
  - `term_id` FK(terms)
  - `lang` TEXT en|ja
  - `surface` TEXT

- `definitions`（そのまま運用）:
  - `id` PK
  - `term_id` FK(terms)
  - `lang` TEXT（ja固定推奨だが拡張可）
  - `text` TEXT（summary を格納）
  - `provider` TEXT
  - `updated_at` DATETIME

正規化/インデックス指針
- `terms.slug` に UNIQUE。
- `term_variants(term_id, lang, surface)` に UNIQUE。
- `term_aliases(term_id, lang, surface, kind)` に UNIQUE 目標。
- `term_tags(term_id, tag_group, tag_value)` に UNIQUE 目標。

正規化・照合ポリシー
- 英語: 小文字化、ハイフン/空白正規化、単複の素朴吸収。
- 日本語: 全角/半角統一、カタカナ正規化、中点・長音のゆらぎ吸収。
- variants は表記ゆれのみ。意味上の別名/略語は aliases に保存。

移行方針
- 目標: `papers/chunks` を保持し、辞書系テーブルのみ再作成。
- 手順案（SQLite）:
  1) 既存の辞書系テーブル（terms/term_variants/definitions/occurrences 等）をダンプ
  2) `papers` と `chunks` を残して schema を v2 に再作成
  3) 旧データのうち必要なものを再インポート（任意）。MVPなら空の辞書から開始可

備考
- C（occurrences）は v2 時点では仕様維持。後段で関係抽出（D3）に合わせて拡張予定。

