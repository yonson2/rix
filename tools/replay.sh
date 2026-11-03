#!/usr/bin/env bash
# Replay check ([072], [075]): seeds 1, 2, 5, 7 and 8 with the scripted line at 30, 60 and
# 120 frames per second, without the renderer. Each seed's RIX_STATE hash must be the same
# at every frame rate and equal its line in tools/replay-hashes.txt, so two CI targets
# that pass give the same game. Seeds 2, 5 and 8 catch a frame-rate dependent ball; 7 has
# an arrow. PROFILE picks the cargo profile (default dev).
set -euo pipefail
cd "$(dirname "$0")/.."
profile=${PROFILE:-dev}
if [ "$profile" = dev ]; then dir=debug; else dir=$profile; fi
cargo build --locked --profile "$profile" --features dev_tools
out=target/replay
rm -rf "$out" && mkdir -p "$out"
for seed in 1 2 5 7 8; do
    for fps in 30 60 120; do
        RIX_SEED=$seed RIX_SHOT_FPS=$fps RIX_SHOT_SCENE=replay RIX_SHOT="$out/unused.png" \
            BEVY_ASSET_ROOT="$PWD" "target/$dir/rix" 2>&1 \
            | grep -o 'RIX_STATE.*' > "$out/$fps.txt" || true
    done
    test -s "$out/30.txt"
    diff "$out/30.txt" "$out/60.txt"
    diff "$out/30.txt" "$out/120.txt"
    echo "seed $seed: $(cat "$out/30.txt")"
    echo "$seed $(cut -d' ' -f4 "$out/30.txt")" >> "$out/hashes.txt"
done
grep -v '^#' tools/replay-hashes.txt | diff - "$out/hashes.txt"
echo "replay: same state at 30, 60 and 120 fps, as in tools/replay-hashes.txt"
