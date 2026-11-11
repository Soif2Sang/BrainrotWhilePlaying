# BrainrotWhilePlaying

TikTok over your game, controlled with your numpad. Windows app built with Tauri and Rust.

![BrainrotWhilePlaying over Valheim](assets/screenshots/valheim-overlay.png)

## Features

- TikTok always visible above your game.
- Numpad controls for scrolling, play/pause, volume, and mute while playing.
- Customizable shortcuts, saved between launches.
- Settings overlay you can show or hide.
- One key cycles through interaction, game with active shortcuts, and keyboard-free mode (only the activation key stays active).
- Freely draggable window, adjustable sizes, and corner positioning.
- Compact view or full TikTok website.

## How to build

Requires Windows, Node.js 22+, stable Rust (MSVC), Microsoft Edge WebView2 Runtime,
and Visual Studio Build Tools with **Desktop development with C++** and a Windows SDK.

```powershell
git clone https://github.com/Soif2Sang/brainrotwhileplaying.git
cd brainrotwhileplaying
npm ci
npm run build
```

The executable is at `src-tauri/target/release/brainrotwhileplaying.exe`.

For development, use `npm run dev`. To run the tests:

```powershell
npm test
cargo test --locked --manifest-path src-tauri/Cargo.toml
```
