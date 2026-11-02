default: dev
dev:
    cargo run --features dev

# Screenshot of a scripted run (one line across the field) into target/shots, then exit ([006]).
shot path="target/shots/game.png":
    mkdir -p "$(dirname '{{path}}')"
    RIX_SEED=1 RIX_SHOT='{{path}}' cargo run --features dev_tools

# Screenshot of the menu into target/shots, then exit ([006]).
shot-menu path="target/shots/menu.png":
    mkdir -p "$(dirname '{{path}}')"
    RIX_SEED=1 RIX_SHOT='{{path}}' RIX_SHOT_SCENE=menu cargo run --features dev_tools
