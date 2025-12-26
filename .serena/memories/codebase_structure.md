# Codebase Structure

## Module Architecture

```
src/
├── main.rs          - Entry point, CLI definition with clap
├── models/
│   └── mod.rs       - Data models (Feed, Article)
├── db/
│   └── mod.rs       - Database operations (SQLite)
├── feed/
│   └── mod.rs       - RSS/Atom fetching and parsing
└── commands/
    └── mod.rs       - CLI command implementations
```

## Module Responsibilities

### main.rs
- Defines CLI structure using `clap` derive macros
- `Cli` struct with `Commands` enum for subcommands
- Entry point `main()` function with `#[tokio::main]`
- Initializes database and dispatches to command handlers
- Commands: Add, Remove, List, Fetch, Articles, Read, Favorite, Unfavorite, Favorites

### models/mod.rs
- **Feed struct**: Stores feed metadata (id, url, title, description, timestamps)
- **Article struct**: Stores article data (id, feed_id, title, url, content, published_at, is_read, is_favorite, created_at)
- Both implement `Debug`, `Clone`, `Serialize`, `Deserialize`
- Constructor methods: `Feed::new()`, `Article::new()`

### db/mod.rs
- **Database struct**: Wraps `rusqlite::Connection`
- Database location: `~/.rustfeed/rustfeed.db`
- **Tables**:
  - `feeds`: id, url (UNIQUE), title, description, created_at, updated_at
  - `articles`: id, feed_id (FK), title, url, content, published_at, is_read, is_favorite, created_at
  - UNIQUE constraint on `(feed_id, url)` to prevent duplicates
  - `ON DELETE CASCADE` for feed deletion
  - Indexes on feed_id, is_read, is_favorite for performance
- **Methods**:
  - `new()`, `init()` - Setup (includes migration for is_favorite column)
  - `add_feed()`, `remove_feed()`, `get_feeds()`, `get_feed()` - Feed CRUD
  - `add_article()`, `get_articles(unread_only, limit, filter)`, `mark_as_read()` - Article CRUD
  - `add_favorite()`, `remove_favorite()`, `get_favorite_articles()` - Favorites management

### feed/mod.rs
- **fetch_feed(url)**: Async function to fetch and parse RSS/Atom feeds
- Uses `reqwest` for HTTP, `feed-rs` for parsing
- Returns `(Feed, Vec<Article>)` tuple
- Articles returned with `feed_id = 0` (caller must set correct ID)
- Supports RSS 0.9/1.0/2.0, Atom 0.3/1.0, JSON Feed
- Includes test: `test_fetch_feed()`

### commands/mod.rs
- Implements all CLI command functions
- Each function takes `&Database` and command-specific parameters
- Uses `colored` crate for terminal output
- **Functions**:
  - `add_feed()` - Fetch and add feed to database
  - `remove_feed()` - Remove feed by ID
  - `list_feeds()` - Display all feeds
  - `fetch_feeds()` - Fetch articles from all feeds (resilient error handling)
  - `show_articles(db, unread_only, limit, filter)` - Display articles with filters (supports keyword filtering)
  - `mark_as_read()` - Mark article as read
  - `add_favorite()` - Add article to favorites
  - `remove_favorite()` - Remove article from favorites
  - `show_favorites()` - Display favorite articles

## Data Flow

1. **User Input** → CLI (main.rs) parses with `clap`
2. **Database Init** → `Database::new()` and `init()`
3. **Command Dispatch** → Pattern match on `Commands` enum
4. **Command Execution** → calls function in `commands/`
5. **Data Operations** → `commands/` coordinates `feed/` and `db/`
6. **Output** → Colored terminal messages to user

## Important Patterns

- **Ownership**: Functions take `&Database`, `&Feed` references to avoid clones
- **Error Context**: Uses `.with_context()` to add meaningful error messages
- **Option Handling**: `map()`, `unwrap_or_else()`, `if let Some()` patterns
- **Async**: Network I/O is async; database operations are sync
- **Resilient Fetching**: `fetch_feeds()` continues on individual feed errors
- **Duplicate Prevention**: `INSERT OR IGNORE` with UNIQUE constraints
