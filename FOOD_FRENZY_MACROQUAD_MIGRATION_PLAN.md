# Food Frenzy Rust Migration Plan + Implementation Notes

## Goal
Migrate gameplay from the prior React/frontend structure into a runnable Macroquad game in this Rust repo while preserving core systems: cooking, customer flow, servings, VIP processing, upgrades, recipes, prestige, and save/load.

## Source references reviewed
- `H:\WebHatchery\game_apps\food_frenzy\README.md` for gameplay loop and mechanics intent.
- `README.md` in this Rust repo root for game context.

## What is implemented
- Added Macroquad simulation entry point in `src/main.rs` with frame loop, input dispatch, game world updates, autosave, and persistence restore.
- Implemented gameplay helper layer in `src/engine/mod.rs` for deterministic calculations and chance constants.
- Implemented immediate-mode UI + hitbox collection in `src/ui/mod.rs` (`serve`, `invite`, station control, upgrades, recipes, prestige, messages).
- Wired persistence and timer updates (`spawn`, `patience`, `satisfaction decay`, trait behaviors, cooking completion).
- Updated `index.html` to match Feast Frenzy identity and Rust WASM entry (`food_frenzy.wasm`).

## Runtime fixes applied during implementation
- Fixed prestige execution from keyboard shortcut (`P`) to apply progression prestige directly.
- Restored proper timer accumulator updates so patience, decay, and trait behaviors progress over real time.
- Removed no-op guest-fed recording in impatience removal logic.
- Removed recursive shadowing bug in serving score helper by using engine helper directly.
- Removed local `random_dish_name` proxy and used engine helper directly.
- Removed double-application of `base_score_multiplier` inside `add_score`.
- Updated scoring calls for recipes and VIP rewards to keep score behavior explicit and avoid multiplier conflicts.

## Known follow-up work (post-implementation)
- Tune tuning values in `assets/data/game_balance.json` after first in-browser test pass.
- Add more detailed UI/UX copy and responsive layout tweaks.
- Add save migration handling if you want backward compatibility for future schema changes.
- Add explicit end-to-end build/run checks for wasm release packaging.

## Files changed
- `src/main.rs`
- `src/engine/mod.rs`
- `src/ui/mod.rs`
- `index.html`

