# Sound sources

Every file under `assets/audio/` has a row here: how it was made and its licence
(AGENTS.md hard rule: no asset ships without a known source and licence). A file made by
a generator keeps its exact request beside this sheet, so it can be made again.

| File | Source | Licence |
|---|---|---|
| `sfx/tap.ogg` | ElevenLabs Sound Effects ([042]), master `audio-src/sfx/tap.wav`, request in `tap.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/line_start.ogg` | ElevenLabs Sound Effects ([042]), master `audio-src/sfx/line_start.wav`, request in `line_start.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/line_hum.ogg` | ElevenLabs Sound Effects ([042]), master `audio-src/sfx/line_hum.wav`, request in `line_hum.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/claim.ogg` | ElevenLabs Sound Effects ([042]), master `audio-src/sfx/claim.wav`, request in `claim.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/eat.ogg` | ElevenLabs Sound Effects ([042]), master `audio-src/sfx/eat.wav`, request in `eat.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/bomb.ogg` | ElevenLabs Sound Effects ([042]), master `audio-src/sfx/bomb.wav`, request in `bomb.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/bounce.ogg` | ElevenLabs Sound Effects ([042]), master `audio-src/sfx/bounce.wav`, request in `bounce.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/death.ogg` | ElevenLabs Sound Effects ([042]), master `audio-src/sfx/death.wav`, request in `death.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/level_clear.ogg` | ElevenLabs Sound Effects ([042]), master `audio-src/sfx/level_clear.wav`, request in `level_clear.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/shot.ogg` | ElevenLabs Sound Effects ([084]), master `audio-src/sfx/shot.wav`, request in `shot.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/hatch.ogg` | ElevenLabs Sound Effects ([084]), master `audio-src/sfx/hatch.wav`, request in `hatch.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/multiplier.ogg` | ElevenLabs Sound Effects ([084]), master `audio-src/sfx/multiplier.wav`, request in `multiplier.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/time_stop.ogg` | ElevenLabs Sound Effects ([084]), master `audio-src/sfx/time_stop.wav`, request in `time_stop.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `sfx/auto_line.ogg` | ElevenLabs Sound Effects ([084]), master `audio-src/sfx/auto_line.wav`, request in `auto_line.json` | ElevenLabs paid plan (Creator): commercial use allowed |
| `music/menu.ogg` | Suno v6 through EvoLink ([043]), "Rix chiptune menu" take `chiptune-menu-1b`, master `audio-src/music/menu.mp3`, request in `menu.json` | Suno: commercial use allowed, Pedro checked the licence himself (D22) |
| `music/game.ogg` | Suno v6 through EvoLink ([043]), "Rix chiptune game" take `chiptune-game-1a`, master `audio-src/music/game.mp3`, request in `game.json` | Suno: commercial use allowed, Pedro checked the licence himself (D22) |

Sound effects ([042]): `tools/elevenlabs.py sfx PROMPT --out audio-src/takes/sfx/NAME --seconds S
--takes 3` makes three takes (`audio-src/takes/` stays out of git). The picked take is copied
to `audio-src/sfx/NAME.wav`; its sidecar records the request, the take it came from and
the encode: the trailing silence trimmed (not for the `line_hum` loop), the peak set to
`encode.peak_db`, then `oggenc -q 5`. The takes were picked by measurement (an instant
start, little dead tail, a level that leaves room for many sounds at once), not by ear.
Pedro then listened to the takes of the [042] sounds on the [043] page (each at its
sound's peak) and picked four others by ear ([087]): `tap-1`, `line_start-3`,
`line_hum-2`, `death-2`. Their sidecars say `picked_by`.

Music ([043], D22): `tools/suno.py` with the briefs in `audio-src/music/briefs.json` makes
two takes per brief (`audio-src/takes/music/`). Pedro picked the style (chiptune) and the
tracks by ear on the listening page. The picked MP3 is copied to `audio-src/music/NAME.mp3`;
its sidecar records the request, the take and the encode: the tail under -50 dB trimmed,
the gain to -16 LUFS, a 10 ms fade in, then `oggenc -q 3` (about 110 kbps).
