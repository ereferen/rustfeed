# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rustfeed is a CLI RSS feed reader written in Rust. It fetches RSS/Atom feeds, stores articles in a local SQLite database, and provides commands to manage feeds and articles.

## Common Commands

### Development
```bash
# Build the project
cargo build

# Build for release (optimized)
cargo build --release

# Run the application
cargo run

# Run with arguments (e.g., add a feed)
cargo run -- add https://blog.rust-lang.org/feed.xml

# Check for errors without building
cargo check

# Run tests
cargo test

# Run a specific test
cargo test test_fetch_feed

# Generate and open documentation
cargo doc --open

# Generate documentation including private items
cargo doc --no-deps --document-private-items
```

### Using the CLI
```bash
# Add a feed
cargo run -- add <url> [--name "Custom Name"]

# List all feeds
cargo run -- list

# Fetch articles from all feeds
cargo run -- fetch

# Show articles
cargo run -- articles [--unread] [--limit 20]

# Mark article as read
cargo run -- read <article_id>

# Remove a feed
cargo run -- remove <feed_id>
```

## Architecture

### Module Structure

The application follows a clean modular architecture with clear separation of concerns:

- **`main.rs`**: Entry point with CLI argument parsing using `clap`. Defines the `Commands` enum with all available subcommands and orchestrates the flow.
- **`models/`**: Core data structures (`Feed` and `Article`) with their constructors. These are serializable with `serde` and represent the domain model.
- **`db/`**: Database layer using SQLite via `rusqlite`. Handles all CRUD operations and persistence. Database file is stored at `~/.rustfeed/rustfeed.db`.
- **`feed/`**: RSS/Atom feed fetching and parsing using `feed-rs` and `reqwest`. Converts external feed formats to internal data models.
- **`commands/`**: Implementation of CLI commands, tying together the other modules and providing user-facing functionality with colored output.

### Data Flow

1. User invokes CLI command → parsed by `clap` in `main.rs`
2. `main.rs` initializes `Database` and calls appropriate function in `commands/`
3. `commands/` functions coordinate between `feed/` (for fetching) and `db/` (for storage)
4. Results are displayed to user with colored output using `colored` crate

### Database Schema

**feeds table**:
- `id` (INTEGER, PRIMARY KEY)
- `url` (TEXT, UNIQUE, NOT NULL)
- `title` (TEXT, NOT NULL)
- `description` (TEXT, nullable)
- `created_at` (TEXT, RFC3339 format)
- `updated_at` (TEXT, RFC3339 format)

**articles table**:
- `id` (INTEGER, PRIMARY KEY)
- `feed_id` (INTEGER, FOREIGN KEY → feeds.id, CASCADE DELETE)
- `title` (TEXT, NOT NULL)
- `url` (TEXT, nullable)
- `content` (TEXT, nullable)
- `published_at` (TEXT, RFC3339 format, nullable)
- `is_read` (INTEGER, 0/1 boolean)
- `created_at` (TEXT, RFC3339 format)
- UNIQUE constraint on `(feed_id, url)` to prevent duplicates

### Key Design Patterns

- **Async/await**: Network operations use `tokio` async runtime for non-blocking I/O
- **Error handling**: Uses `anyhow::Result` for error propagation with context
- **Option types**: Leverages Rust's `Option<T>` for nullable fields instead of null pointers
- **Ownership**: Functions take references (`&Database`, `&Feed`) to avoid unnecessary clones
- **DateTime handling**: All timestamps stored as RFC3339 strings in SQLite, parsed to `chrono::DateTime<Utc>` in Rust

### Important Implementation Details

- The `feed::fetch_feed()` function returns articles with `feed_id = 0`; the caller must set the correct `feed_id` before inserting into the database
- Articles use `INSERT OR IGNORE` to prevent duplicate entries based on the `(feed_id, url)` unique constraint
- The codebase includes extensive rustdoc comments in Japanese for learning purposes
- Error handling in `fetch_feeds` is resilient: individual feed failures don't stop processing other feeds
- Feed deletion cascades to articles automatically via `ON DELETE CASCADE`

## Code Style Notes

- The codebase contains comprehensive rustdoc comments written in Japanese as educational material
- Comments explain Rust concepts like ownership, borrowing, lifetimes, pattern matching, and traits
- When adding new code, match the existing rustdoc style if contributing to this learning-focused codebase
