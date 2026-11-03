# Credits

Rix is a game by Pedro. It is inspired by Qix (Taito, 1981) and Xonix (1984).

The game shows these credits on its CREDITS screen (`src/screens/credits.rs`). Keep the two
in step.

## Assets

Every asset that ships has a known source and licence (AGENTS.md hard rule).

| Asset | Source | Licence |
|---|---|---|
| Font `assets/fonts/main.ttf` | "balatro" by localthunk, built with FontStruct: <https://fontstruct.com/fontstructions/show/2326420/balatro> | SIL Open Font License 1.1, text in `assets/fonts/OFL.txt` |
| Sound effects `assets/audio/sfx/` | Made with ElevenLabs Sound Effects; one row per file in `audio-src/sources.md` | ElevenLabs paid plan: commercial use allowed |
| Music `assets/audio/music/` | Made with Suno v6 through EvoLink; one row per file in `audio-src/sources.md` | Suno: commercial use allowed (Pedro checked the licence, D22) |
| Bomb image `assets/items/bomb_basic.png` | Drawn by `tools/draw-bomb.py` ([098]) | GPL-3.0, as the code |
| Shaders `assets/shaders/` | Written for Rix from scratch, with other shaders as references (Pedro, [099]) | GPL-3.0, as the code |

## Code

Rix is free software under the GNU General Public License 3.0 (`LICENSE`, D35). It is
written in Rust with the [Bevy](https://bevy.org) engine and the
[avian](https://github.com/Jondolf/avian) physics engine (both MIT or Apache-2.0). The
licence of every crate in the shipped build, with its text, is in
`assets/licenses/third-party.md`, which ships with the app. `just licences` makes it again
after a dependency change.
