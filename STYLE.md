# Style guide

This project follows three published guides. Where they overlap, the stricter
rule wins.

- [The Rust Style Guide](https://doc.rust-lang.org/style-guide/) for layout,
  enforced by `rustfmt` on edition 2024.
- [The Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html)
  for naming (C-CASE, C-CONV, C-GETTER), typed errors (C-GOOD-ERR), and using
  types rather than `bool`s, tuples or magic numbers to carry meaning
  (C-CUSTOM-TYPE).
- [The Linux kernel's Rust coding guidelines](https://docs.kernel.org/rust/coding-guidelines.html)
  for `unsafe`, panics, and lint suppression.

Lints are configured in `Cargo.toml` (`clippy::pedantic` plus a few
restriction lints).

## Checks

A change is done when all of these pass without warnings:

```sh
cargo fmt --check
cargo clippy --all-targets
cargo xwin clippy --target x86_64-pc-windows-msvc   # Windows-only code paths
cargo test
```

## Rules

**Comments**

- No comments or doc comments in the code. Names, types and small functions
  carry the meaning. The only exception is the `reason` on an `#[expect]`.

**Safety and panics**

- No `unwrap()`. Use `?`, handle the `None`/`Err`, or use `expect("…")` with a
  message stating the invariant that makes failure impossible.
- A mutex guarding plain data recovers from poisoning instead of panicking
  (see `audio::lock`). Audio callbacks must never panic.
- Silence a lint with `#[expect(lint, reason = "…")]`, never `#[allow]`, so a
  suppression that is no longer needed turns into a warning. For code that
  only warns on some platforms, use `cfg_attr(…, expect(…))`.

**Types**

- Errors are typed. Network failures are `net::Error`, and messages carry
  `Result<T, net::Error>`. Convert to `String` only where text is displayed.
- Use a named struct instead of a tuple whose fields have meaning
  (`lastfm::Session`, `article::Span`, `state::Notice`).
- Use an enum instead of a `bool` or number with more than an on/off meaning
  (`api::Reaction`, `widgets::CellWidth`, `bevel::Kind`).
- Form validation belongs to the form state (`RegisterForm::validate`), not
  to the update handler.

**Naming**

- No abbreviations beyond the universal ones (`id`, `url`, `msg`, `px`).
  Write `button`, not `btn`, and `window`, not `win`.
- Messages are named for what happened (`WindowClosed`, `FavoriteAdded`) or
  what the user asked for (`OpenWindow`, `Remove`).
- The Plaza API functions are named `fetch_*` for reads and
  verb-first for actions (`add_favorite`, `delete_account`).

## Architecture

The app uses iced's Elm architecture. Data flows one way:

```
main.rs          boot, subscriptions, window titles
message.rs       Msg: every event the app handles
state.rs         Plaza: the model, plus per-window state and form validation
update/          update(): applies a Msg to Plaza, returns follow-up Tasks
views/           view(): renders Plaza; custom widgets live alongside
window.rs        WindowKind: every secondary window, its size and title
```

Everything that touches the outside world is isolated behind a small
interface and reports back only through messages:

```
net.rs              shared HTTP agent, error type, off-thread execution
api.rs              Plaza REST API
lastfm.rs           Last.fm auth and scrobbling
audio/              stream download (pipe.rs), device (output.rs), decode loop
media_controls.rs   OS media keys and now-playing widgets
discord.rs          Discord Rich Presence worker
config.rs           settings file
platform.rs         OS shims: allocator tuning, opening URLs
fonts.rs            bundled fonts and fallback setup
```

Dependencies point downward. `views` and `update` depend on `state` and
`message`, never on each other. Service modules never depend on `state`,
`update` or `views`; the ones that emit events only know `message::Msg`.
