# Quickwit Development Guide

## Build & Development Commands
- Build: `make build` or `make -C quickwit build`
- Run tests: `make test-all` (all tests), `cargo nextest run <test-name>` (specific test)
- Lint & fix: `make fix` (fixes issues), `make fmt` (formats code)
- Run clippy: `cargo clippy --workspace --all-features --tests`
- Check format: `cargo +nightly fmt --check`
- Build UI: `make build-ui`

## Code Style Guidelines
- **Naming**: Use descriptive names even if long. Function names should clearly indicate purpose.
- **Types**: Add explicit type annotations for strategic variables to aid reviewers.
- **Error Handling**: Prefer early returns over nested if/else. Use `debug_assert` for invariants.
- **Comments**: Use rustdoc style. Comments should convey intent, context, not implementation details.
- **Readability**: Prioritize "proofreadability" - code that's easy to verify for correctness.
- **Formatting**: Use `cargo +nightly fmt` for consistent formatting.
- **Macros/Generics**: Use sparingly as they can hurt readability and compile time.
- **Error Messages**: Concise, lowercase (except proper names), no trailing punctuation.
- **Logging**: Prefer structured logging: `warn!(value=x, "message")` over templating.
- **Testing**: Unit tests can test private functions. Use proptests where beneficial.
- **Async Code**: Async code should block for at most 500 microseconds.