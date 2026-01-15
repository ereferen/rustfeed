# CLAUDE.md

rustfeed - CLI/TUI対応のRSS/Atomフィードリーダー（Rust製）

## クイックスタート

```bash
# ビルド
cargo build

# テスト
cargo test

# CLI実行
cargo run --bin rustfeed-cli -- <command>

# TUI実行
cargo run --bin rustfeed-tui
```

## よく使うコマンド

```bash
# フィード追加
cargo run --bin rustfeed-cli -- add https://example.com/feed.xml

# 記事取得
cargo run --bin rustfeed-cli -- fetch

# 記事一覧
cargo run --bin rustfeed-cli -- articles --unread
```

## 詳細ドキュメント

詳細な規約・ガイドラインは `.claude/rules/` を参照：

- [01-ai-instructions.md](.claude/rules/01-ai-instructions.md) - AIへの作業指示
- [02-code-style.md](.claude/rules/02-code-style.md) - コードスタイル規約
- [03-workflow.md](.claude/rules/03-workflow.md) - 開発ワークフロー
- [04-architecture.md](.claude/rules/04-architecture.md) - アーキテクチャ方針

## Claude Code Skills

このプロジェクトには、Claude Code専用のSkillsが用意されています：

- **rustfeed-architecture**: プロジェクト構造・アーキテクチャガイド
- **rustfeed-quality**: コード品質チェック（fmt, clippy, test, doc）
- **rustfeed-feed-ops**: フィード管理・記事操作コマンド

詳細は [.claude/SKILLS.md](.claude/SKILLS.md) を参照してください。

## プロジェクト構成

```
rustfeed/
├── rustfeed-core/    # 共有ライブラリ（models, db, feed, config）
├── rustfeed-cli/     # CLIバイナリ
└── rustfeed-tui/     # TUIバイナリ（ratatui使用）
```

## データベース

SQLite（`~/.rustfeed/rustfeed.db`）

**feeds table**: id, url, title, description, created_at, updated_at

**articles table**: id, feed_id, title, url, content, published_at, is_read, is_favorite, created_at

## 技術スタック

- 非同期: tokio
- エラー: anyhow
- DB: rusqlite
- フィード: feed-rs
- HTTP: reqwest
- CLI: clap
- TUI: ratatui
