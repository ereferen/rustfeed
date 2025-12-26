# Suggested Commands

## Building and Running

### Build Commands
```bash
# Check for compilation errors without building
cargo check

# Build in debug mode
cargo build

# Build in release mode (optimized)
cargo build --release

# Clean build artifacts
cargo clean
```

### Running the Application
```bash
# Run in development
cargo run

# Run with arguments (examples)
cargo run -- add https://blog.rust-lang.org/feed.xml
cargo run -- add https://example.com/feed.xml --name "My Feed"
cargo run -- list
cargo run -- fetch
cargo run -- articles
cargo run -- articles --unread
cargo run -- articles --unread --limit 10
cargo run -- articles --filter "rust"
cargo run -- articles --filter "rust,cargo" --unread -l 10
cargo run -- read <article_id>
cargo run -- favorite <article_id>
cargo run -- unfavorite <article_id>
cargo run -- favorites
cargo run -- favorites -l 10
cargo run -- remove <feed_id>
```

## Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_fetch_feed

# Run tests with output
cargo test -- --nocapture

# Run tests in verbose mode
cargo test -- --test-threads=1 --nocapture
```

## Documentation
```bash
# Generate and open documentation
cargo doc --open

# Generate docs with private items (for comprehensive view)
cargo doc --no-deps --document-private-items

# Generate docs without dependencies
cargo doc --no-deps
```

## Code Quality
```bash
# Format code (if rustfmt is configured)
cargo fmt

# Check formatting without modifying
cargo fmt --check

# Run clippy linter (if configured)
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::all
```

## Dependency Management
```bash
# Add a new dependency
cargo add <crate_name>

# Remove a dependency
cargo remove <crate_name>

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated  # requires cargo-outdated to be installed
```

## Development Workflow
```bash
# Quick check cycle during development
cargo check

# Before committing changes
cargo fmt
cargo clippy
cargo test
cargo build
```
