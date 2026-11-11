# License scope and third-party material

The original application source code and documentation are provided under the
[MIT license](LICENSE). Dependencies retain their own licenses; the exact
dependency versions are recorded in `package-lock.json` and `src-tauri/Cargo.lock`.

TikTok is a third-party service. Its website, videos, account data, trademarks,
and other content are not part of this repository's MIT license. This project
is independent of TikTok and is not endorsed by it.

The screenshot in `assets/screenshots/` illustrates the running application over
Valheim. The game and TikTok content depicted belong to their respective owners
and are not included in the source code's MIT license.

## Mascot and branding

The artwork in `assets/*.png`, `ui/logo.png`, and `src-tauri/icons/` was generated
and edited with OpenAI ImageGen. The prompts are recorded alongside the source
artwork in `assets/`. The mascot is inspired by the existing Tung Tung Tung Sahur
character; [the character's published reference](https://www.mementumlab.com/wiki-tung-tung)
identifies its creator as Noxa / @noxaasht.

The mascot artwork is excluded from the code's MIT license. This project does
not grant rights to third-party characters, names, or trademarks, or claim
ownership of them. Use your own artwork when redistributing a rebranded build
unless you have the rights needed to reuse this mascot. No affiliation with
the character's creator or representatives is implied.

To replace the artwork, run `npx tauri icon path/to/your-logo.png --output
src-tauri/icons`, then copy `src-tauri/icons/128x128.png` to `ui/logo.png`.
The Windows build uses `icon.ico`, `icon.png`, and `128x128.png` from that directory.
