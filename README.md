# rustfeed

A CLI RSS reader written in Rust.

## Overview

rustfeed is a command-line RSS feed reader designed to help you efficiently collect and manage information from multiple sources.

## Features (Planned)

### Phase 1 (MVP)
- [ ] Add/remove RSS feeds
- [ ] Fetch and list articles
- [ ] Mark articles as read/unread
- [ ] Local SQLite database for persistence

### Phase 2
- [ ] Keyword filtering
- [ ] Favorites/bookmarks
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

# Mark as read
rustfeed read <article_id>
```

## License

MIT License
