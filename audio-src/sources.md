# Sound sources

Every file under `assets/audio/` has a row here: how it was made and its licence
(AGENTS.md hard rule: no asset ships without a known source and licence). A file made by
a generator keeps its exact request beside this sheet, so it can be made again.

| File | Source | Licence |
|---|---|---|
| `sfx/tap.ogg` | Placeholder ([041]): `ffmpeg -f lavfi -i "aevalsrc=0.5*sin(2*PI*880*t)*exp(-t*40):s=44100:d=0.12" -c:a vorbis -strict -2 -ac 2` | Own work |
| `music/menu.ogg` | Placeholder ([041]): `ffmpeg -f lavfi -i "aevalsrc=0.25*sin(2*PI*220*t)*(0.6+0.4*sin(2*PI*0.5*t)):s=44100:d=4" -c:a vorbis -strict -2 -ac 2` | Own work |
| `music/game.ogg` | Placeholder ([041]): `ffmpeg -f lavfi -i "aevalsrc=0.25*sin(2*PI*330*t)*(0.6+0.4*sin(2*PI*2*t)):s=44100:d=4" -c:a vorbis -strict -2 -ac 2` | Own work |

The placeholders are sine tones; [042] (sound effects) and [043] (music) replace them.
