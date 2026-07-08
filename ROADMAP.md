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

- [ ] **Pacing retune** (data-only): `visits_until_ready` scales by tier
      (2 for tier 1, up to 5 at tier 4), faster spawns, 3 starting tables.
      All in `assets/data/*.json`, no hardcoding.
- [ ] **Onboarding system** — scripted first-guest tutorial as a data-driven
      step machine (`state/tutorial.rs` + `assets/data/tutorial.json`):
      cook → serve → fatten → invite → first processing. Skippable, resumable.
      (~600 LOC)
- [ ] **Guest state HUD** — per-guest fattening / satisfaction / patience
      meters and a "plump and ready" callout (`ui/guest_status.rs`). You can't
      play around state you can't see. (~300 LOC)
- [ ] **Last Meal Lounge dramatization** — a dedicated processing sequence:
      transition, animation timeline, meat-payoff reveal, wrong-note audio
      sting (`ui/lounge.rs` + `engine/cinematic.rs`). The review calls this the
      single highest-leverage addition. (~800 LOC)
- [ ] **Floating gain numbers + transaction log** for every cash/score/meat
      change (`ui/floaters.rs`). Check `macroquad-toolkit` first — a floating-
      number widget is a plausible toolkit upgrade candidate. (~250 LOC)
- [ ] **Economy legibility** — score becomes a visible prestige progress bar
      with a target; cash and meat get iconed, explained tooltips; conversion
      ratios surfaced in UI instead of living as magic numbers. (~250 LOC)

**Exit check:** replay the owner's `feedback.md` playtest script; a new player
must reach a processing inside 5 minutes and be able to say what meat is for.

## Phase 2 — Depth & tension (review: High, ~2.0k LOC)

Review goal: *real decisions and kitchen tension in the moment-to-moment.*

- [ ] **Dish freshness/spoilage** — cooked dishes decay on the pass; serving
      fresh pays more (`engine/freshness.rs`, tuning in JSON). Kills
      fire-and-forget cooking. (~300 LOC)
- [ ] **Trait counterplay engine** — traits become telegraphed situations with
      player answers and first-encounter hints (Fox steals unless the pass is
      cleared; Monkey throws food below a satisfaction threshold the code
      already checks but never explains; wanderers need reseating). Data-driven
      behavior table (`engine/traits.rs` + JSON). (~700 LOC)
- [ ] **Combo/chain juice** — visible combo meter, streak rewards, room-flow
      bonus for a fully served floor (Diner Dash lesson). (~250 LOC)
- [ ] **Café specialization** — pick a house style (e.g. Carnivore's Corner /
      Sweet Parlor / Rustic Hearth) with real trade-offs: which clientele it
      attracts, dish bonuses, spoilage rates (`state/specialization.rs` +
      JSON). Makes runs diverge. (~450 LOC)
- [ ] **Aspirational clientele board** — the 13-type ladder shown as a goal
      board with silhouettes, unlock costs, and meat sources (front-load the
      first unlocks in data). (~300 LOC)

## Phase 3 — Content & session structure (review: Medium, ~2.3k LOC)

Review goal: *sustain the players Phases 1–2 retain.*

- [ ] **Named regulars** — persistent, recognizable guests with names,
      personality lines, and per-guest memory; processing a regular pays more
      and cuts deeper (the review: "emotional weight (and darkness)").
      (`state/regulars.rs` + name/personality JSON) (~500 LOC)
- [ ] **Day/shift framing** — soft day cycle with an end-of-day ledger
      (earnings, guests fattened, meat gained, next unlock progress) and
      optional day goals (`state/day_cycle.rs` + `ui/day_summary.rs`).
      (~650 LOC)
- [ ] **Dining events** — a small data-driven event system that creates
      situations, not modifiers: dinner rush, a health-inspector visit to
      hide the Lounge from, a guest who suspects something, a VIP glutton.
      (`engine/events.rs` + JSON) (~550 LOC)
- [ ] **Content expansion** (mostly JSON + small code): recipes 6 → 12+ with
      earlier first unlock, 3–4 new traits wired to the counterplay engine,
      milestone goals between prestige walls, achievements to ~20. (~300 LOC)
- [ ] **Prestige rework** — first wall lowered/curved, prestige grants a
      choice of permanent meta-unlocks (keep one clientele tier, start
      specialized) instead of a bare multiplier. (~250 LOC)

## Phase 4 — Presentation & tone (review: Large, ~1.5k LOC + assets)

Review goal: *make the café-hiding-a-butchery identity felt and shareable.*

- [ ] **Art/tone pass** — build on the pixel sprite sheet: warm café palette
      with subtly wrong details, ambient dining-room life (idle chatter
      bubbles, chef busywork), décor that darkens as tiers rise. (~600 LOC)
- [ ] **Audio system** — toolkit-first: cozy ambience loop, cooking/serving
      SFX, and the processing sting from Phase 1 grown into a full mixing
      layer with settings. (~450 LOC)
- [ ] **Save robustness** — versioned persistence with migration for all the
      new state (regulars, specialization, day cycle, tutorial progress).
      (~300 LOC)
- [ ] **Capture scenes** — register `title`, `dining_rush`, `lounge`,
      `day_summary`, `clientele_board` with the capture harness; refresh
      `catalog_thumbnail.png`; rewrite the standing-visible README feature
      list. (~150 LOC)

---

## Verification cadence (every phase)

1. `cargo clippy --all-targets --all-features -- -D warnings` + `cargo test`.
2. `..\macroquad-toolkit\scripts\capture_ui.ps1 -Scenes <new scenes>` for
   visual verification of new UI.
3. `.\publish.ps1` from `food_frenzy/` after each phase lands (per AGENTS.md).
4. Playtest against `feedback.md`'s original complaints — the review treats
   those notes as the acceptance bar.

## Ledger

| Phase | New systems | Est. LOC | Running total |
|---|---|---:|---:|
| Start | — | — | ~6.0k |
| 0 | tests, restructure | +0.3k | ~6.3k |
| 1 | tutorial, guest HUD, lounge cinematic, floaters, legible economy | +2.5k | ~8.8k |
| 2 | freshness, trait counterplay, combo, specialization, goal board | +2.0k | ~10.8k |
| 3 | regulars, day cycle, events, content, prestige rework | +2.3k | ~13.1k |
| 4 | tone pass, audio, save versioning, captures | +1.5k | ~14.6k |

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
