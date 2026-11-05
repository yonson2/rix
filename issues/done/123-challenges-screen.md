# [123] The CHALLENGES screen

**Status:** done
**Milestone:** M9
**Depends on:** [109], [114], [121], [122]
**Created:** 2026-11-05
**Closed:** 2026-11-05

## Problem
D39 (Pedro): a challenge section with a list of special seeds. A tap plays one, and its
code also plays from PLAY SEED.

## Approach
- A CHALLENGES button on the title opens a list: each curated challenge with its name, a
  one-line rule text and the local best. A tap plays it.
- Each curated challenge is a fixed code (field seed and rules), so every player gets the
  same field.
- PLAY SEED takes challenge codes and shows the challenge name, or "CUSTOM CHALLENGE" for
  a code that is not on the list.
- The game over screen shows the challenge name and its best. The share text gives the
  name and the code.
- `Save` keeps the best per challenge code. A challenge run counts toward the lifetime
  stats, the games played and the medals, not the best level, the best score or the main
  board ([109] Q3).
- Some challenges open with games played (`Stats.games`, [114]; [109] Q4). A locked entry
  says "OPENS AT 20 GAMES (12 / 20)". A shared or typed code always plays. The RECORDS
  screen's next unlock ([113]) lists the next challenge too. The schedule (the agent's
  pick, between the look tiers of [114]): open at the start: 60 SECONDS, BALL PIT, BARE
  HANDS; 10 games: OVERDRIVE; 15: SWARM; 20: BULLET HELL; 30: SPLIT DECISION; 40: THREE
  MINUTES; 60: GREEDY; 75: DEEP END.
- The first list ([109] Q6: it stays):
  - BALL PIT: balls only, 2x enemies.
  - BULLET HELL: shooters only.
  - SPLIT DECISION: splitters only.
  - SWARM: 3x enemies.
  - OVERDRIVE: 1.5x speed, more points.
  - 60 SECONDS: the most points in one minute.
  - THREE MINUTES: the most points in three minutes.
  - GREEDY: 90 % to clear a level.
  - BARE HANDS: no items.
  - DEEP END: start at level 10.

## Verification
`cargo test` passes (every listed code parses to its rules; the names); a `challenges` shot
scene shows the list; `just shot-flow` style run: a tap starts the challenge.

## Notes
- From Pedro's message (2026-11-05, D39).
- Result: `src/game/challenges.rs` (`CHALLENGES`: name, rule line, `Rules`, a fixed
  40-bit field, `opens_at`) and `src/screens/challenges.rs` (`Screen::Challenges`). The
  title's PLAY SEED row is now two half buttons: CHALLENGES and PLAY SEED (no new row:
  the title has no room for one at 450 x 800). A row is 400 x 52 (about 45 pt): name in
  yellow, rule in white, "BEST N" at the right; a locked row is a gray box, not a button.
- `Save::challenge_bests` keeps the best per code (also for a custom code);
  `record_run` reads the rules from the seed: a challenge leaves `Save::best` alone and
  counts its levels cleared from its start level (DEEP END starts at 10). `record_level`
  skips challenge runs (`rules::normal_rules`), so DEEP END opens no level looks.
  Games played, stats, medals and secrets count ([109] Q3).
- Not in the Approach, added to make it work: RETRY and the pause RESTART play a
  challenge again (`setup::retry_seed`; a normal run still gets a new field), and a game
  that opens a challenge says "NEW CHALLENGE: X / PLAY IT IN CHALLENGES" on the game
  over screen, as a new skin does. No challenge opens at a look's game count (a test).
- Game over: the name in cyan over the title (`NOTICE_Y` 315 to 330 to leave it room).
  Share: "I scored N on BALL PIT in Rix! Seed CODE". PLAY SEED: the name in cyan under
  the code, "CUSTOM CHALLENGE" for other rules. RECORDS: "NEXT CHALLENGE: X (G / N)"
  under the next look; the medals moved down 22.
- The main board is not built yet: the rule is in the [068] notes.
- Evidence: `cargo test` 195 passed; new tests: `every_listed_code_parses_to_its_rules_and_name`,
  `a_code_off_the_list_is_a_custom_challenge`, `games_played_open_the_challenges_in_order`,
  `no_challenge_opens_with_a_look`, `a_tap_on_an_open_row_plays_its_challenge` (an
  `Activate` on the BALL PIT row sets its `Run`, `Rules` and `Screen::Gameplay`),
  `a_locked_row_says_when_it_opens`, `the_rows_fit_above_the_back_button`,
  `a_challenge_run_keeps_its_own_best_and_counts_from_its_start_level`,
  `a_retry_plays_a_challenge_again_and_a_normal_run_on_a_new_field`,
  `a_challenge_code_shows_its_name`, and the share and records asserts.
  `RIX_SHOT_SCENE=challenges` (a save with 12 games and a BALL PIT best): four open rows,
  six locked with their progress, "BEST 48,210" on BALL PIT. The flow scene on the BALL
  PIT seed: 2 enemies at level 1 before and after RETRY. A 60 SECONDS game over shows
  the name over "TIME!". `just replay` passes; default `just shot` unchanged (md5
  e170997305a806320c6f9f967cf3cf5b); `just shot-flow` as before; fmt, both clippy runs
  and `cargo build` clean.
