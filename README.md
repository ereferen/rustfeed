# rustfeed

A CLI RSS reader written in Rust.

## Overview

rustfeed is a command-line RSS feed reader designed to help you efficiently collect and manage information from multiple sources.

## Features

### Phase 1 (MVP) ✅
- [x] Add/remove RSS feeds
- [x] Fetch and list articles
- [x] Mark articles as read/unread
- [x] Local SQLite database for persistence

### Phase 2 ✅
- [x] Keyword filtering
- [x] Favorites/bookmarks
- [x] Export to various formats (JSON/Markdown)
- [x] Configuration file support
- [x] Feed filtering
- [x] Feed management (rename, categorize, prioritize)
- [x] Read management (batch mark, toggle)
- [x] Article search with date range filtering

### Phase 3 (In Progress)
- [x] TUI (Terminal UI)
- [x] Tauri GUI

## Installation

### Option 1: Docker (Recommended for Development with Claude Code)

Docker環境を使用すると、環境構築が簡単で、Claude Codeをすぐに使い始められます。

**APIキー不要**: ブラウザでログインするだけでClaude Codeを使用できます。

```bash
# Clone the repository
git clone https://github.com/ereferen/rustfeed.git
cd rustfeed

# 【推奨】自動セットアップとテスト
make validate   # 設定ファイル検証
make test-env   # 環境の自動構築とテスト

# または手動セットアップ
make setup      # .env作成 + ビルド（APIキーなしでもOK）
make up         # コンテナ起動
make shell      # コンテナに接続
make claude     # Claude Code起動 → 初回はブラウザでログイン
```

詳細は [DOCKER.md](./DOCKER.md) を参照してください。

### Option 2: Local Installation

```bash
# Clone the repository
git clone https://github.com/ereferen/rustfeed.git
cd rustfeed

# Build CLI
cargo build --release -p rustfeed-cli

# Build TUI
cargo build --release -p rustfeed-tui

# Run CLI
cargo run --bin rustfeed-cli

# Run TUI
cargo run --bin rustfeed-tui
```

### GUI (Tauri + React)

The GUI requires additional dependencies:

```bash
# Install Tauri CLI
cargo install tauri-cli

# Install frontend dependencies
cd apps/rustfeed-gui
npm install

# Run in development mode
cargo tauri dev

# Build for production
cargo tauri build
```

**System Requirements for GUI:**
- Node.js 18+
- Rust 1.70+
- Platform-specific dependencies:
  - **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Microsoft Visual Studio C++ Build Tools

## Usage

```bash
# Add a feed
rustfeed add <url>

# List feeds
rustfeed list
rustfeed list --category "Tech"         # Filter by category

# Feed management
rustfeed rename <feed_id> "New Name"    # Rename feed
rustfeed update-url <feed_id> <new_url> # Update feed URL
rustfeed set-category <feed_id> "Tech"  # Set category
rustfeed set-priority <feed_id> 10      # Set priority (higher = first)
rustfeed info <feed_id>                 # Show feed details

# Fetch articles
rustfeed fetch

# Show articles
rustfeed articles
rustfeed articles --unread              # Show unread articles only
rustfeed articles --filter "rust"       # Filter by keyword
rustfeed articles --filter "rust,cargo" # Filter by multiple keywords (OR)
rustfeed articles --feed 2              # Show articles from feed ID 2 only
rustfeed articles --after "2025-01-01"  # Articles from Jan 1, 2025 onwards
rustfeed articles --before "2025-12-31" # Articles before Dec 31, 2025
rustfeed articles --last-days 7         # Articles from the past 7 days
rustfeed articles --last-weeks 2        # Articles from the past 2 weeks
rustfeed articles --filter "rust" --unread -l 10 --last-days 7  # Complex filters

# Search articles
rustfeed search "rust async"            # Full-text search
rustfeed search "docker" --unread -l 10 # Search unread articles
rustfeed search "kubernetes" --after "2025-01-01"  # Search with date filter

# Read management
rustfeed read <article_id>              # Mark single article as read
rustfeed mark-all-read                  # Mark all articles as read
rustfeed mark-all-read --feed 2         # Mark all from feed 2 as read
rustfeed mark-all-read --before "2025-01-01"  # Mark old articles as read
rustfeed mark-unread <article_id>       # Mark as unread
rustfeed mark-unread --feed 2           # Mark all from feed 2 as unread
rustfeed mark-unread --all              # Mark all as unread
rustfeed toggle-read <article_id>       # Toggle read/unread status

# Favorites
rustfeed favorite <article_id>    # Add to favorites
rustfeed unfavorite <article_id>  # Remove from favorites
rustfeed favorites                # Show favorite articles

# Export articles
rustfeed export                         # Export to JSON (default)
rustfeed export --format markdown       # Export to Markdown
rustfeed export --favorites             # Export favorites only
rustfeed export --unread -l 50          # Export 50 unread articles
rustfeed export > backup.json           # Save to file
```

## Configuration

rustfeed supports configuration via TOML file at `~/.config/rustfeed/config.toml`:

```toml
[general]
default_limit = 20           # Default article display limit
show_unread_only = false     # Show only unread articles by default
disabled_feeds = [3, 5]      # Hide articles from these feed IDs

[display]
date_format = "%Y-%m-%d"     # Date format string (chrono format)
show_description = true      # Show feed descriptions

[database]
path = "~/.rustfeed/rustfeed.db"  # Database file path
```

If the configuration file doesn't exist, default values are used.
```

## License

MIT License
