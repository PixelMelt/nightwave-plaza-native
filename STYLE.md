# Style

Follows the [Rust Style Guide](https://doc.rust-lang.org/style-guide/), the [API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html), and the [kernel Rust guidelines](https://docs.kernel.org/rust/coding-guidelines.html). `clippy::pedantic` enforces it; `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` must pass clean.

- No comments.
- No `unwrap()`; use `?` or `expect("invariant")`.
- `#[expect(lint, reason = "...")]`, never `#[allow]`.
- Typed errors (`net::Error`), not `String`.
- Named structs and enums instead of tuples, `bool` flags and magic numbers.
- No abbreviations: `button`, not `btn`.
- Elm architecture: `message` → `update` → `state` → `views`. Services (`api`, `audio`, `lastfm`, ...) talk back only through `Msg`.
