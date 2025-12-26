# Task Completion Checklist

When completing a task in this project, follow these steps:

## 1. Code Quality Checks

### Format Code
```bash
cargo fmt
```
- Ensures consistent code formatting
- Should be run before committing

### Run Linter (if available)
```bash
cargo clippy
```
- Checks for common mistakes and potential improvements
- Address any warnings unless there's a good reason not to

## 2. Testing

### Run Tests
```bash
cargo test
```
- Ensure all tests pass
- Add new tests for new functionality
- Update existing tests if behavior changed

### Manual Testing (if applicable)
```bash
# Build and run the application
cargo build
cargo run -- <test commands>
```
- Test the specific feature you worked on
- Verify edge cases and error handling

## 3. Documentation

### Check Documentation
```bash
cargo doc --no-deps --document-private-items
```
- Ensure rustdoc comments are present for new public items
- Follow the existing Japanese documentation style
- Explain Rust concepts where appropriate (this is an educational codebase)

### Update README if needed
- Add information about new features
- Update usage examples if commands changed

## 4. Build Verification

### Final Build
```bash
# Check for warnings
cargo build

# Build release version to ensure optimization works
cargo build --release
```
- Should complete without errors or warnings

## 5. Git Workflow (if applicable)

### Stage and Commit
```bash
git add <files>
git commit -m "descriptive message"
```
- Write clear, concise commit messages
- Follow the project's commit message style

## 6. Special Considerations

### Database Changes
- If schema changed, ensure migration strategy
- Test with existing database files
- Document any breaking changes

### Async Code
- Ensure proper error handling with `.await?`
- Verify tokio runtime is properly configured

### Dependencies
- If added new dependencies, verify they're necessary
- Check license compatibility (project is MIT)
- Update Cargo.toml with appropriate features

## Quick Pre-Commit Checklist
```bash
cargo fmt
cargo clippy
cargo test
cargo build
```

All four commands should complete successfully before committing changes.
