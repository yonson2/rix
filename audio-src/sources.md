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
| `music/menu.ogg` | Placeholder ([041]): `ffmpeg -f lavfi -i "aevalsrc=0.25*sin(2*PI*220*t)*(0.6+0.4*sin(2*PI*0.5*t)):s=44100:d=4" -c:a vorbis -strict -2 -ac 2` | Own work |
| `music/game.ogg` | Placeholder ([041]): `ffmpeg -f lavfi -i "aevalsrc=0.25*sin(2*PI*330*t)*(0.6+0.4*sin(2*PI*2*t)):s=44100:d=4" -c:a vorbis -strict -2 -ac 2` | Own work |

The music placeholders are sine tones; [043] (music) replaces them.

Sound effects ([042]): `tools/elevenlabs.py sfx PROMPT --out audio-src/takes/sfx/NAME --seconds S
--takes 3` makes three takes (`audio-src/takes/` stays out of git). The picked take is copied
to `audio-src/sfx/NAME.wav`; its sidecar records the request, the take it came from and
the encode: the trailing silence trimmed (not for the `line_hum` loop), the peak set to
`encode.peak_db`, then `oggenc -q 5`. The takes were picked by measurement (an instant
start, little dead tail, a level that leaves room for many sounds at once), not by ear.
