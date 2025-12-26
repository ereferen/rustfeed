# Project Overview

## Purpose
rustfeed is a CLI RSS feed reader written in Rust. It helps users efficiently collect and manage information from multiple RSS/Atom feed sources.

## Main Features
- Add/remove RSS feeds
- Fetch and list articles from feeds
- Mark articles as read/unread
- Favorites/bookmarks for important articles
- Keyword filtering (search in title and content)
- Feed filtering (show/hide specific feeds)
- Export to JSON/Markdown formats
- Configuration file support (TOML)
- Local SQLite database for persistence
- Colored terminal output for better UX

## Development Status
Phase 1 (MVP) completed ✅. Phase 2 completed ✅:
- Phase 1 (MVP) ✅: 
  - ✅ Add/remove RSS feeds
  - ✅ Fetch and list articles
  - ✅ Mark articles as read/unread
  - ✅ Local SQLite database
- Phase 2 (Completed) ✅: 
  - ✅ Keyword filtering
  - ✅ Favorites/bookmarks
  - ✅ Export to various formats (JSON/Markdown)
  - ✅ Configuration file support
  - ✅ Feed filtering (--feed flag, disabled_feeds)
- Phase 3 (Planned): 
  - ⏳ TUI (Terminal UI)
  - ⏳ Tauri GUI (optional)

## Recent Updates (2025-12-26)
- Added export functionality for articles (Issue #5)
  - JSON and Markdown export formats
  - Filter by favorites, unread status
  - Customizable export limits
- Added configuration file support (Issue #4)
  - TOML-based config at ~/.config/rustfeed/config.toml
  - XDG Base Directory Specification compliant
  - Settings for default_limit, disabled_feeds, date_format, etc.
- Added feed filtering feature
  - --feed flag to show articles from specific feed
  - disabled_feeds in config to hide feeds
  - --feed overrides disabled_feeds

## Repository
- GitHub: https://github.com/ereferen/rustfeed
- License: MIT
- Author: ereferen
