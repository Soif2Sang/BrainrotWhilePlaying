# Contributing

Bug reports and focused pull requests are welcome. English and French are both fine.

## Run locally

Follow the Windows prerequisites in the [README](README.md), then:

```powershell
npm ci
npm run dev
```

The frontend is plain HTML/CSS/JavaScript in `ui/`. Native window behavior and
global shortcuts live in `src-tauri/src/main.rs`; shortcut validation is in
`bindings.rs`, TikTok player commands in `player.js`, and compact-page styling
in `overlay.js`.

## Before submitting

```powershell
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --locked --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml -- -D warnings
npm run build
```

For window, focus, shortcut, or TikTok changes, also describe the manual checks:
interaction mode, game mode, overlay open/close, shortcut editing, and navigation
while another app has focus. Automated DOM tests do not prove compatibility
with TikTok's live page or a game's input handling.

Keep changes focused. Include a regression test when fixing non-trivial behavior.
Do not grant native permissions to the remote TikTok WebView: native commands
must remain restricted to the trusted local settings WebView. Keep the lockfiles
committed and avoid committing generated builds, logs, account data, or cookies.

By contributing original code, you agree to license that contribution under the
project's MIT license. See [NOTICE.md](NOTICE.md) before contributing or reusing branding.
