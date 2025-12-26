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

### Phase 3
- [ ] TUI (Terminal UI)
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
rustfeed articles --unread              # Show unread articles only
rustfeed articles --filter "rust"       # Filter by keyword
rustfeed articles --filter "rust,cargo" # Filter by multiple keywords (OR)
rustfeed articles --feed 2              # Show articles from feed ID 2 only
rustfeed articles --filter "rust" --unread -l 10  # Combine filters

# Mark as read
rustfeed read <article_id>

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
