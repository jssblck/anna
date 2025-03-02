# CLAUDE.md - Anna Project Guidelines

## Build/Test Commands
- Build project: `cargo build`
- Run tests: `cargo test`
- Run specific test: `cargo test test_name`
- Run tests with nextest: `cargo nextest run`
- Format code: `rustfmt`
- Lint code: `cargo clippy`
- Check dependencies: `cargo machete` and `cargo upgrade`

## Code Style Guidelines
- Follow Rust API Guidelines
- Use functional style code when possible
- Prefer type inference for assignments
- Add MPL 2.0 license headers to all source files using plain comments
- Don't add comments that duplicate code; DO explain WHY we do something
- Prefer `expect` over `unwrap`; proper error handling over both
- Use `color_eyre` for error handling
- Follow newtype pattern for public interfaces
- Use `String::from()` not type annotations with `.into()`
- Add integration tests under `tests/it/`
- Prefer named module files over `mod.rs`
- Don't edit `Cargo.toml` directly - use `cargo edit`
- Use blank lines between enum variants and struct fields