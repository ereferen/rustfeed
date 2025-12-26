# Code Style and Conventions

## Documentation Style
- **Language**: Rustdoc comments are written in Japanese as educational material
- **Detail Level**: Very comprehensive, explaining Rust concepts (ownership, borrowing, lifetimes, pattern matching, traits, etc.)
- **Format**: Uses standard Rust doc comments (`///` for items, `//!` for modules)
- **Structure**: Includes sections like `# 引数` (Arguments), `# 戻り値` (Return values), `# 例` (Examples), `# エラー` (Errors)

## Naming Conventions
- **Modules**: Snake_case (e.g., `commands`, `db`, `feed`, `models`)
- **Structs**: PascalCase (e.g., `Feed`, `Article`, `Database`)
- **Functions**: Snake_case (e.g., `fetch_feed`, `add_feed`, `mark_as_read`)
- **Variables**: Snake_case (e.g., `feed_id`, `is_read`, `published_at`)

## Code Organization
- Each module has a `mod.rs` file with comprehensive module-level documentation
- Public API clearly separated from private implementation
- Uses `#[allow(dead_code)]` for items planned for future use

## Error Handling
- Uses `anyhow::Result<T>` for error propagation
- Adds context to errors with `.with_context()` for better debugging
- Uses `?` operator for error propagation
- Graceful degradation: individual failures don't stop overall processing

## Async Patterns
- Network operations are async using `tokio`
- Functions marked with `async fn` where I/O is involved
- Uses `.await` for async operations

## Type Safety
- Extensive use of `Option<T>` for nullable values instead of null pointers
- Explicit type conversions (e.g., `bool` to `i32` for SQLite)
- DateTime always stored as RFC3339 strings in database, parsed to `DateTime<Utc>` in Rust

## Comments
- Inline comments explain "why" rather than "what"
- Educational comments explain Rust concepts throughout the codebase
- Japanese and some English mixed in comments
