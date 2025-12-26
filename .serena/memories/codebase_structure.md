# Codebase Structure

## Workspace Architecture (v0.5.0+)

The project uses a Cargo workspace with three crates:

```
rustfeed/
├── Cargo.toml                 # Workspace configuration
├── crates/
│   ├── rustfeed-core/         # Shared library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs         # Re-exports (Database, Feed, Article, AppConfig)
│   │       ├── models.rs      # Feed, Article structs
│   │       ├── db.rs          # Database operations (SQLite)
│   │       ├── feed.rs        # RSS/Atom fetching and parsing
│   │       └── config.rs      # Configuration management
│   │
│   ├── rustfeed-cli/          # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs        # Entry point + clap CLI definition
│   │       └── commands.rs    # CLI command implementations
│   │
│   └── rustfeed-tui/          # TUI binary
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs        # Entry point
│           ├── app.rs         # Application state
│           ├── event.rs       # Event handling (placeholder)
│           └── ui/
│               └── mod.rs     # UI rendering (ratatui)
```

## Binary Commands

- `cargo run -p rustfeed-cli -- <command>` - Run CLI
- `cargo run -p rustfeed-tui` - Run TUI

## Module Responsibilities

### rustfeed-core
Shared library providing:
- **models.rs**: Feed and Article data structures with serde support
- **db.rs**: SQLite database operations (CRUD for feeds/articles)
- **feed.rs**: Async RSS/Atom feed fetching using reqwest + feed-rs
- **config.rs**: TOML configuration at ~/.config/rustfeed/config.toml

### rustfeed-cli
CLI interface with subcommands:
- Feed: add, remove, list, rename, update-url, set-category, set-priority, info
- Article: fetch, articles, search, read, mark-all-read, mark-unread, toggle-read
- Favorites: favorite, unfavorite, favorites
- Export: export (json/markdown)

### rustfeed-tui
TUI interface using ratatui:
- Two-pane layout (feeds + articles)
- Keyboard navigation (j/k, h/l, Tab)
- Read/favorite toggle support
