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

### Phase 2
- [ ] Keyword filtering
- [x] Favorites/bookmarks
- [ ] TUI (Terminal UI)

### Phase 3
- [ ] Export to various formats
- [ ] Tauri GUI (optional)

## Installation

```bash
# Clone the repository
git clone https://github.com/ereferen/rustfeed.git
cd rustfeed

# Build
cargo build --release

# Run
cargo run
```

## Usage

```bash
# Add a feed
rustfeed add <url>

# List feeds
rustfeed list

# Fetch articles
rustfeed fetch

# Show articles
rustfeed articles
rustfeed articles --unread  # Show unread articles only

# Mark as read
rustfeed read <article_id>

# Favorites
rustfeed favorite <article_id>    # Add to favorites
rustfeed unfavorite <article_id>  # Remove from favorites
rustfeed favorites                # Show favorite articles
```

## License

MIT License
