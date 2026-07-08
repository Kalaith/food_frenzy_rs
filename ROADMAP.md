# Feast Frenzy — Standing Climb Roadmap (2026-07-08)

Goal: execute the `game_review.md` (2026-07-07) roadmap in full, taking the game
from **rank 23 (last)** to **at least rank 10** in `standing.md`.

## What rank 10 actually measures

The standing ranks by *MVP scale and implementation complexity* — Rust LOC,
system count, data/assets, and managed game state. Calibration against the
current table:

| Rank | Game | src LOC | Files |
|---:|---|---:|---:|
| 9 | `iron_fauna` (new entry) | ~13.4k | 52 |
| 10 | `nightmare_shift` | ~14.1k | 38 |
| 11 | `kaiju_sim` | ~13.9k | 57 |
| 12 | `ai_defense` | ~12.2k | 30 |
| 13 | `carriage_run` | ~11.2k | 28 |
| 16 | `dungeon_core` | ~10.1k | 46 |
| **23** | **`food_frenzy`** | **~6.0k** | **23** |

Two lessons from recent movement:

1. **System density beats raw LOC.** `iron_fauna` entered at 9 with a mid-pack
   line count because it carries an unusually high system count.
   `dungeon_core` holds 16 on only 10.1k LOC for the same reason.
2. **The rank is a byproduct, not the target.** Every line must earn its place
   as a real system the review already calls for. Padding would be caught by
   the "apparent complexity" read — and violates the no-unused-code standard.

**Target: ~13–15k LOC across ~45–55 files with 8–10 nameable new systems**,
which lands in the 10–11 band on size and above it on density. Rankings are
relative — peers move too — so the buffer above the current rank-10 line
matters.

The review's verdict frames all of it: *the bones are good; the game fails on
communication and feel.* Everything below is the review's own Phase 1–4 plan,
sequenced and sized. Nothing from its **Avoid** list (multiplayer, idle
automation, more currencies, branching story, twitch difficulty) appears here.

---

## Phase 0 — Foundations (prep, ~0.3k LOC net)

Hygiene before growth, per repo standards:

- [x] Migrate `src/ui/mod.rs` → `src/ui.rs` and `src/engine/mod.rs` →
      `src/engine.rs` (no new `mod.rs` files; don't leave both present).
- [x] Pre-split the two files nearest the limit before piling features onto
      them: `simulation.rs` → orchestrator + `spawning`/`guests`/`traits`;
      `ui/dining.rs` → tables/guests + `dining/room` (floor, fixtures, lounge).
- [x] Reconcile the name everywhere — audited: player-facing copy is already
      "Feast Frenzy" throughout; `food_frenzy`/`FOOD_FRENZY` remain as the
      repo slug and env-var prefix by repo convention. No changes needed.
- [x] Add `#[test]`s that parse every JSON asset + cross-reference checks
      (customer types ↔ meats ↔ recipes ↔ unlock costs, upgrade effect keys
      the engine actually reads) — `src/data.rs`.
- [x] Decide the README stance consciously: kept the "wholesome café" cover
      story as deliberate misdirection, recorded in a README comment.

## Phase 1 — Land the hook (review: Critical, ~2.5k LOC)

Review goal: *a first-time player reaches a dramatized first processing in
~5 minutes and understands the loop.*

- [x] **Pacing retune** (data-only): `visits_until_ready_by_tier` [2,3,4,5]
      via `engine::visits_until_ready_for`, spawns 15s → 10s, 3 initial
      spawn delays, 3 starting tables.
- [x] **Onboarding system** — 8-step data-driven tutorial
      (`state/tutorial.rs` + `assets/data/tutorial.json` +
      `ui/tutorial_panel.rs`): cook → carry → serve → fatten → ready →
      first processing → economy explainer. Skippable, resumes from save.
- [x] **Guest state HUD** — `ui/guest_status.rs`: satisfaction + patience
      bars, fattening pips, pulsing "PLUMP & READY" callout, and a hover
      panel that explains every number.
- [x] **Last Meal Lounge dramatization** — `state/cinematic.rs` (pure,
      tested timeline: escort → curtain → quiet beat → reveal) +
      `ui/lounge.rs` overlay; world pauses, input blocks, payoff panel
      shows meat/renown/cash; `lounge` capture scene added. Audio sting
      deferred to the Phase 4 audio system (no sound assets exist yet).
- [x] **Floating gain numbers** — `state/floaters.rs` + `ui/floaters.rs`;
      serving, departures, walkouts, processing, crafting, upgrades, and
      prestige all spawn rising gain/loss numbers. (Toolkit had toast
      notifications but no world-anchored floaters — noted as a future
      toolkit upgrade candidate.)
- [x] **Economy legibility** — renamed/iconed header tiles with hover
      explanations for all six, prestige progress bar inside the Renown
      tile, new Larder (total meat) tile.

**Exit check:** replay the owner's `feedback.md` playtest script; a new player
must reach a processing inside 5 minutes and be able to say what meat is for.

## Phase 2 — Depth & tension (review: High, ~2.0k LOC)

Review goal: *real decisions and kitchen tension in the moment-to-moment.*

- [x] **Dish freshness/spoilage** — `engine/freshness.rs` + `PlatedDish`
      aging (legacy-save compatible): fresh dishes pay a +25% bill bonus,
      spoiled dishes are discarded off the pass; kitchen rows show a live
      freshness countdown. Tuning in `game_balance.json`.
- [x] **Trait counterplay engine** — telegraphed trait situations
      (`simulation/traits.rs` rework + `assets/data/trait_behaviors.json`):
      fox steal / monkey tantrum / wandering arm a visible warning with a
      countdown and only fire if the player doesn't answer (serve the fox,
      feed the monkey, land a course on the wanderer). One-time
      first-encounter hints per trait, persisted in progression.
- [x] **Combo/chain juice** — floor combo meter showing the live renown
      multiplier, streak cash bonuses every 5th combo, and a full-house
      renown bonus when every table's order completes at once.
- [x] **Café specialization** — Carnivore's Corner / Sweet Parlor / Rustic
      Hearth (`assets/data/specializations.json`), chosen in a modal after
      the first processing; effects feed the same `get_effect` accumulator
      as upgrades (validated by the same test); prestige resets the choice.
- [x] **Aspirational clientele board** — full 4-tier ladder overlay
      (`ui/clientele_board.rs`) with trait identities, meat yields, unlock
      costs, and in-place Attract buttons; cow unlock front-loaded 6 → 4
      pig-meat; `clientele_board` + `specialization` capture scenes added.

## Phase 3 — Content & session structure (review: Medium, ~2.3k LOC)

Review goal: *sustain the players Phases 1–2 retain.*

- [x] **Named regulars** — 40-name pool + 5 personality archetypes
      (`assets/data/regulars.json`); guests carry a personality (arrival
      flavor line), earn a gold "R" badge and 1.5× Lounge yield at 3
      satisfied visits, and their personality's farewell line becomes the
      dark beat of the processing reveal.
- [x] **Day/shift framing** — `state/day_cycle.rs` (tested) + on-floor day
      clock + `ui/day_summary.rs` closing ledger (cash, renown, served,
      lost, processed, meat, fresh, best combo) with a "tomorrow's goal"
      line; the world pauses until the player opens the next day;
      `day_summary` capture scene added.
- [x] **Dining events** — `simulation/events.rs` +
      `assets/data/dining_events.json`: one weighted event per day (dinner
      rush, health inspector locking the Lounge, incognito critic, generous
      evening) with an on-floor banner and countdown.
- [x] **Content expansion** — recipes 6 → 12 with data-driven unlock
      requirements (first unlock front-loaded to 2 pigs); 3 new passive
      traits wired through the behavior table (Big Tipper deer, Tastemaker
      cat, Gourmand bear); achievements 11 → 20, all wired to live
      counters; day-ledger goal line gives near-term milestones.
- [x] **Prestige rework** — first wall 50k → 12k renown with a 1.6×
      per-level curve (`engine::prestige_requirement`); prestiging opens a
      perk choice (keep clientele, keep specialization, war chest, stocked
      cellar) in `ui/prestige_modal.rs`.

## Phase 4 — Presentation & tone (review: Large, ~1.5k LOC + assets)

Review goal: *make the café-hiding-a-butchery identity felt and shareable.*

- [x] **Art/tone pass** — `ui/ambience.rs`: personality-driven idle chatter
      bubbles over seated guests, steam puffs on working stations, and a
      creeping red-dark room tint that deepens as the clientele ladder
      climbs (the "subtly wrong" tone cue).
- [x] **Audio system** — 7 procedurally generated SFX clips in
      `assets/sounds/` (cook, plate, serve, cash, day chime, event, and the
      Lounge's wrong-note sting), loaded via the toolkit's
      `load_sound_from_pack_or_file`; gameplay queues `SfxCue`s that the
      app drains (`src/audio.rs`); sound toggle added to Settings;
      macroquad `audio` feature enabled.
- [x] **Save robustness** — `SAVE_VERSION` 1 → 2 with an explicit
      `migrate()` step on load and a regression test that loads a real
      pre-roadmap v1 fixture (bare-string dishes, no tutorial/day/regular
      state) and verifies nothing is lost.
- [x] **Capture scenes** — 8 scenes registered (`title`, `settings`,
      `gameplay`, `lounge`, `specialization`, `clientele_board`,
      `day_summary`, `dining_rush`); `catalog_thumbnail.png` refreshed from
      the title capture; README rewritten with the full (cozy-framed)
      feature list.

---

## Verification cadence (every phase)

1. `cargo clippy --all-targets --all-features -- -D warnings` + `cargo test`.
2. `..\macroquad-toolkit\scripts\capture_ui.ps1 -Scenes <new scenes>` for
   visual verification of new UI.
3. `.\publish.ps1` from `food_frenzy/` after each phase lands (per AGENTS.md).
4. Playtest against `feedback.md`'s original complaints — the review treats
   those notes as the acceptance bar.

## Ledger

| Phase | New systems | Actual LOC | Running total |
|---|---|---:|---:|
| Start | — | — | ~6.0k |
| 0 | tests, restructure | +0.3k | 6.3k |
| 1 | tutorial, guest HUD, lounge cinematic, floaters, legible economy | +1.2k | 7.5k |
| 2 | freshness, trait counterplay, combo, specialization, goal board | +1.2k | 8.7k |
| 3 | regulars, day cycle, events, content, prestige rework | +1.1k | 9.8k |
| 4 | tone pass, audio, save versioning, captures | +0.4k | 10.2k |

Actuals ran leaner than the estimates — the systems landed at full scope but
denser (49 files, ~10.2k LOC + ~1.1k lines of JSON content + 7 sound assets).
That is a `dungeon_core`-profile entry (10.1k LOC, 46 files, holds rank 16 on
system density), now carrying 14 nameable systems added this cycle. Realistic
landing: **rank 11–13 on this snapshot** — a 10–12 place climb — with rank 10
reachable next cycle by growing content breadth (more customer types, decor
variants, day-goal variety) rather than new systems.

End state: ~14.6k LOC, ~50 files, and a standing blurb that reads like
`iron_fauna`'s — tutorialized fatten-and-process loop with a cinematic payoff,
trait counterplay, café specialization, named regulars, a day cycle with
ledger summaries, dining events, meta-unlock prestige, and full audio — on top
of the existing 13-type meat-web economy. That is a rank ~9–11 profile on both
size and system count, with buffer for peer movement.

## Risks

- **Peers move.** `dungeon_core` and `carriage_run` are actively climbing; the
  ~14.6k target includes buffer above today's rank-10 line, but re-check
  `standing.md` calibration after Phase 2.
- **Density over padding.** If a phase runs long, cut estimated LOC, never
  add filler — the standing explicitly reads *apparent complexity*, and the
  review's Avoid-list discipline is what keeps the game good while it grows.
- **File limit pressure.** `simulation.rs` and `ui/dining.rs` absorb most new
  mechanics; Phase 0's pre-split plus extraction-per-system keeps everything
  under the 800-line hard limit.
