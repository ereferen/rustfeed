# Technology Stack

## Language
- Rust 2021 Edition

## Key Dependencies

### CLI & Terminal
- **clap 4** (with derive features) - CLI argument parser
- **colored 2** - Terminal output decoration

### Async Runtime
- **tokio 1** (with full features) - Async runtime for non-blocking I/O

### HTTP & Feed Parsing
- **reqwest 0.11** (with json features) - HTTP client
- **feed-rs 1** - RSS/Atom feed parser (supports RSS 0.9/1.0/2.0, Atom 0.3/1.0, JSON Feed)

### Database
- **rusqlite 0.31** (with bundled features) - SQLite database interface
- Database location: `~/.rustfeed/rustfeed.db`

### Data Handling
- **serde 1** (with derive features) - Serialization/deserialization
- **serde_json 1** - JSON support
- **chrono 0.4** (with serde features) - Date/time handling

### Error Handling
- **anyhow 1** - Flexible error handling with context
- **thiserror 1** - Custom error types

### Utilities
- **dirs 5** - Home directory detection

### Development Dependencies
- **tokio-test 0.4** - Testing utilities for async code
