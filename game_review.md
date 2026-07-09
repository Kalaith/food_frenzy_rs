# Feast Frenzy — Design Review

*Senior design / systems review. Prepared 2026-07-07. Based on source (`src/`), data (`assets/data/`), screenshots, README, and the owner's `feedback.md` playtest notes.*

---

# 1. Project Overview

## Project Name

**Feast Frenzy** (repo/slug: `feast_frenzy`; earlier working title "Feast Frenzy"). The two names should be reconciled — pick one and use it everywhere.

## Genre

Real-time restaurant/café management sim with an incremental-progression (tycoon/prestige) backbone. Single-screen, mouse-driven, WebGL + native.

## Core Concept

The public README describes a wholesome cooking game: *"prepare house recipes, serve hungry guests, keep the dining room moving."* **That is a cover story.** The actual game — visible in the data, the code, and the owner's own pitch — is a **cute-macabre "fatten and process" loop**:

> "Feed your clients, fatten them up and use the new premium ingredients to attract wealthier clients." — `feedback.md`

The real loop is:
- Anime **animal-girls** (Pig, Cow, Sheep, Rabbit, Cat, Fox, Bear…) walk in as guests.
- You cook and serve them across **repeated visits**, fattening them (feeding capacity grows, "deliciousness" rises).
- After enough visits (`visits_until_ready = 5`) a guest becomes "plump and ready." You invite them to the **Last Meal Lounge** (a VIP table), where they are **processed into meat**.
- That meat is the premium ingredient used to **attract higher-tier clientele** and **craft high-value recipes** — which in turn feed a bigger, richer clientele. Ascend a 4-tier ladder of 13 customer types, chasing score, and **prestige** at 50,000 score for permanent multipliers.

**What makes it different:** the transgressive tonal inversion. It looks like a cozy café; it is quietly a cannibal butchery dressed in pastel. That "cute + macabre" collision (the owner's words) is the entire reason this project is worth continuing.

**Target player:** fans of dark-comedy / cozy-horror management games — the *Ravenous Devils*, *Do Not Feed the Monkeys*, adult-indie-itch.io audience — not the *Overcooked* family crowd the README currently implies.

## Current State

**Early development / playable vertical slice.**

Reasons:
- The full economic chain exists in code (cook → serve → pay → fatten → process → meat → attract/craft → upgrade → prestige) and runs end-to-end. This is past "prototype."
- But it is **thin and unbalanced**: 2 tables, 4 dishes, a 15s spawn timer, and a 5-visit fattening gate make the first meaningful action (processing one guest) take a very long, low-agency grind.
- The signature hook is **invisible in play** — no art, audio, or narrative sells the macabre premise. The UI is placeholder monospace on a black grid (see screenshots).
- The owner's playtest (`feedback.md`) found a total showstopper — *"delivering 4 meals to 2 patrons earnt 0"* — that has since been partially addressed (guests now pay on departure, `simulation.rs::update_departures`), confirming the build is still stabilizing core economy.

It is a working skeleton of a genuinely interesting game, not yet a fun one.

---

# 2. Core Gameplay Analysis

## Main Gameplay Loop

**Moment-to-moment:** `Select station → Cook dish → Carry ready dish → Serve seated guest their ordered course → Guest pays & leaves → repeat`

**Meta loop:** `Fatten guest over 5 visits → Invite to Last Meal Lounge → Process into meat → Spend meat to attract new clientele / craft recipes → Spend cash on upgrades → accumulate score → Prestige → repeat stronger`

Evaluation:

- **Is it clear?** The *cooking/serving* micro-loop is clear (the UI even prints "click to carry" / "click Serve"). The *meta* loop is **not** clear in play — nothing explains that repeat visits fatten a guest, what "deliciousness" does, or that meat gates all progression. A new player will cook, serve, and wonder what the point is (exactly what the owner's playtest reported).
- **Is it satisfying?** Partially. Cooking/serving has tactile feedback. But there is **no tension**: dishes never spoil while waiting (*"no issue leaving them to wait"* — playtest), only 2 tables, and cooking is fire-and-forget. The satisfying "payoff" moment — processing a guest — is buried behind minutes of repetition.
- **Meaningful decisions?** **Weak.** With 4 dishes, 2 tables, and 3 cook slots, there is almost no scheduling pressure or trade-off. You rarely choose between competing good options; you just execute the obvious one.
- **Variety?** Low early, moderate late. 13 customer types with 8 special traits is real content, but the pacing hides almost all of it behind the meat gate.
- **Long-term motivation?** The prestige + upgrade + achievement scaffolding exists, but 50,000 score is a distant wall and the ride there is repetitive. Long-term motivation is currently *structural, not felt.*

**Verdict:** The loop is complete but under-tuned and under-communicated. The fun is theoretically present and practically absent.

---

# 3. Existing Systems Review

## Cooking / Stations

**Purpose:** Produce the 4 dish colors (Forager Toasts/blue, Hearth Broth/green, Butcher's Roast/yellow, Velvet Sweets/red) that satisfy guest orders.

**Current implementation:** 4 stations, per-dish cook times 3–7s, a 3-dish `cooking_slots_limit`, click-to-start / click-to-carry, keyboard 1–4 shortcuts. Cooked dishes queue on the station indefinitely.

**Strengths:** Clean, readable, immediately understandable. Color-coded courses map cleanly to guest orders. The chef-actor movement adds a little life.

**Weaknesses:** No spoilage, no burn, no capacity pressure → cooking is a solved, tensionless chore. Fire-and-forget removes all skill. With only 2 guests, you rarely need more than one dish at a time.

**Improvement ideas:** Add dish freshness/spoilage decay (a cooked dish loses satisfaction value or is discarded over time) to create the kitchen tension the game lacks. Tie cook order to demand. — **Impact: High. Cost: Small.**

## Serving & Courses

**Purpose:** Deliver ordered courses (1–3 per guest) to seated guests; preferred dishes pay/score double.

**Current implementation:** `roll_order` builds a 1–3 course order weighted toward the guest's preferred colors; serving marks courses done, adds to the bill, feeds combo/chain.

**Strengths:** Preferred-dish targeting is a genuine (small) decision and the strongest existing "read the guest" mechanic. Course structure gives orders shape.

**Weaknesses:** With 2 guests the routing is trivial. The "preferred" signal is shown as small colored dots that are easy to miss; the payoff (2× bill/score) isn't communicated.

**Improvement ideas:** Surface preference clearly and reward correct reads with visible juice. Later, add order timers per course. — **Impact: Medium. Cost: Small.**

## Fattening → Last Meal Lounge → Processing (the signature system)

**Purpose:** The core fantasy and the sole source of meat, which gates all vertical progression.

**Current implementation:** Guests must be fully served across `visits_until_ready = 5` visits; then `invite_customer_to_vip` (85% accept) removes them, yields `times_fed + deliciousness`-scaled meat, and awards a big score+cash burst. `special_table_process_time = 3000ms` cooldown.

**Strengths:** This is the game's soul and it *works mechanically*. Yield scaling by how fattened/delicious a guest is is a smart, thematic reward curve. Traits (`high_yield`, `multiplies_on_process`) add spice.

**Weaknesses — this is where the project is losing the most value:**
1. **The payoff is invisible and undramatic.** The single most important, most transgressive moment in the game is a ticker line and a table cooldown. No animation, no reveal, no *frisson*.
2. **The 5-visit gate is brutally slow** given 15s spawns, 65% return chance, and 2 tables. Reaching your *first* processing — the moment that unlocks the entire game — can take many minutes of pure repetition.
3. **Players don't understand it.** Nothing teaches that repeat feeding is the path, so the central verb is discoverable only by accident.

**Improvement ideas:** (a) Make the Lounge a real cinematic beat — a dedicated animation/transition, sound, and a visible "meat gained" payoff. (b) Cut `visits_until_ready` to 2–3 for the first few guests (or scale it up as tiers rise) so the hook lands inside the first few minutes. (c) Add a clear per-guest "fattening meter" so the player sees progress toward readiness. — **Impact: Game-changing. Cost: Medium.**

## Clientele Ladder (Customer Types & Traits)

**Purpose:** Vertical progression — spend meat to unlock richer guests, who yield rarer meat.

**Current implementation:** 13 types, 4 profile tiers, unlock costs paid in specific meats (e.g. Bear needs cow+goat+pig meat). 8 special traits (`low_appetite`, `high_yield`, `can_wander`, `can_steal_food`, `throws_food`, `multiplies_on_process`, `fast_spoilage`, `can_eat_waste`).

**Strengths:** This is the game's best-developed content and its clearest long-term goal. The meat-web (each tier's meat feeds the next tier's unlock) is an elegant crafting-economy spine. Traits give types identity.

**Weaknesses:** Traits are mostly *passive annoyances applied to the player* (fox steals your dish, monkey throws it) with **no counterplay** — they read as random punishment, not as characterization the player can respond to. Tier gating is so slow that most players will never see tiers 3–4.

**Improvement ideas:** Give traits player-facing meaning and counterplay (e.g. Fox: guard the pass, or serve fast; Monkey: keep satisfaction above threshold — which the code already checks but never explains). Show the ladder as an aspirational "menu of guests" so players *want* the next tier. — **Impact: High. Cost: Medium.**

## Economy (Cash / Score / Meat)

**Purpose:** Three intertwined resources: **cash** (buys upgrades), **score** (prestige threshold + achievements), **meat** (attract + craft).

**Current implementation:** Cash from bills+tips on departure, plus fractions of processing (`awarded/5`) and crafting (`awarded/4`). Score from serving, processing, crafting, with combo & prestige multipliers.

**Strengths:** Multiple sinks and sources create a real economy. Combo/chain multipliers reward flow.

**Weaknesses:** **The three currencies are muddled and unexplained.** Score's only real function is a distant prestige gate; players can't tell why they'd want score vs cash vs meat. The conversion ratios (`/4`, `/5`, `*10`) are opaque magic numbers with no in-game legibility. The owner's playtest literally couldn't tell money was being earned.

**Improvement ideas:** Consider collapsing to **two** legible currencies (cash + meat) and making "score" purely a prestige/leaderboard progress bar with a visible target. Show every transaction as a floating number. — **Impact: High. Cost: Small–Medium.**

## Recipes

**Purpose:** High-value meat sinks that grant score, cash, and future feeding capacity.

**Current implementation:** 6 recipes unlocked by processing counts (e.g. "Bacon Ramen" after 5 pigs); crafting consumes meat for a points burst + capacity bonus.

**Strengths:** Good secondary meat sink; thematically rich names ("Rainbow Stew" from one of each animal). Ties capacity growth to engagement.

**Weaknesses:** Almost entirely back-loaded — locked behind processing counts most players won't reach. Crafting is a menu click with no craft fantasy. Overlaps conceptually with attracting clientele (both are "spend meat") without a clear reason to choose one.

**Improvement ideas:** Bring the first recipe unlock much earlier as a tutorializing reward. Differentiate recipes from unlocks (e.g. recipes = immediate cash, unlocks = long-term growth). — **Impact: Medium. Cost: Small.**

## Upgrades

**Purpose:** Cash sink for incremental efficiency (cook speed, patience, yield, combo, extra table, spawn rate, decay, recipe value, capacity).

**Current implementation:** 9 upgrade tracks with exponential cost growth, applied via a generic `get_effect` accumulator.

**Strengths:** Broad, data-driven, clean implementation. Covers every pressure point. "Dining Room Expansion" (more tables) is the meaningful one.

**Weaknesses:** Most are flat, boring percentage bumps players buy without feeling. With so few tables early, upgrades barely register. No visual change from upgrading.

**Improvement ideas:** Foreground the table-count upgrade (the one that actually changes play); make the rest fewer but chunkier. — **Impact: Medium. Cost: Small.**

## Prestige

**Purpose:** Long-term reset for permanent multipliers.

**Current implementation:** At 50,000 score, reset everything for prestige points → a permanent score multiplier (`+0.03` each) and starting cash.

**Strengths:** Standard, functional incremental hook; gives a top-level goal.

**Weaknesses:** 50,000 is a very distant first wall on top of a slow early game. Nothing on the road to it feels like build-up. Prestige resets clientele unlocks — potentially deflating after a long grind.

**Improvement ideas:** Lower/curve the first prestige; make prestige a celebrated milestone with a meta-unlock, not just a multiplier. — **Impact: Medium. Cost: Small.**

## Presentation / UI / Feel

**Purpose:** Communicate state and sell the fantasy.

**Current implementation:** Three-panel monospace layout (Kitchen / Dining room / Café plan) on a black grid; tiny character sprites; a scrolling ticker for all feedback. No audio system.

**Strengths:** Functional, legible, everything on one screen. The animal-girl sprite sheet exists and is charming.

**Weaknesses:** **This is the second-biggest problem after pacing.** The presentation actively hides the game's identity — it reads as a spreadsheet, not a darkly cute café. No audio, no juice, no reaction to the macabre core. The empty dining grid feels lifeless.

**Improvement ideas:** A focused art/audio/juice pass built entirely around the "cozy-café-hiding-a-butchery" tone. This is where the USP becomes *felt*. — **Impact: Game-changing. Cost: Large.**

---

# 4. Similar Games & Lessons

## Ravenous Devils

The single closest comparable: a dark restaurant sim where you murder customers and cook them into dishes for the next customers. **Similar:** the exact butchery-restaurant fantasy and tonal inversion. **Does better:** it *commits* — the killing and cooking are the on-screen, animated core verbs; the horror is explicit and stylish. **Lesson:** don't hide your hook. The processing moment must be the game's showpiece, not a ticker line. **Don't copy:** its linear story structure — Feast Frenzy's incremental/roguelite economy is a fine alternative spine.

## Cook, Serve, Delicious!

**Similar:** frantic cook-and-serve micro-loop, order matching, upgrade economy. **Does better:** kitchen *tension* — timing, spoilage, combo pressure, and escalating rushes make execution a skill. **Lesson:** add spoilage and rush pressure so cooking stops being fire-and-forget. **Don't copy:** its punishing twitch difficulty; keep Feast Frenzy cozier.

## Diner Dash

**Similar:** seat guests, read patience, serve, turn tables. **Does better:** legible patience/mood telegraphing and the satisfying rhythm of chaining a full room. **Lesson:** make patience and satisfaction *visible* per guest, and reward chaining a busy room — but that requires more than 2 tables. **Don't copy:** its shallow long-term progression.

## Idle/Tycoon incrementals (Adventure Capitalist et al.)

**Similar:** prestige, multipliers, exponential upgrades, meta reset. **Does better:** every number is legible and every purchase visibly accelerates you; the dopamine of watching growth is constant. **Lesson:** make the economy transparent and every gain visible (floating numbers, progress bars to the next unlock). **Don't copy:** full idle automation — Feast Frenzy's active cooking is its differentiator; don't automate the fun away.

**Core lesson across all four:** commit to the tone (Ravenous Devils), add kitchen tension (CSD), make guests legible (Diner Dash), make the economy transparent (incrementals).

---

# 5. Feature Improvement List

## Critical Improvements

| Priority | Feature | Description | Player Benefit | Dev Cost |
|---|---|---|---|---|
| Critical | Fix early pacing | Lower `visits_until_ready` to 2–3 early, raise spawn rate / starting tables to 3 | Player reaches the first "process" payoff in minutes, not tens of minutes | Small |
| Critical | Teach the meta loop | Tutorial/onboarding + per-guest fattening meter + "readiness" callout | Player understands *why* they cook and what the goal is | Medium |
| Critical | Dramatize processing | Dedicated Last Meal Lounge animation/sound/reveal + visible meat payoff | The signature moment finally lands emotionally | Medium |
| Critical | Legible economy | Floating gain numbers; clarify cash vs score vs meat; consider dropping to 2 currencies | Player sees progress and cause→effect | Small–Medium |

## High Value Improvements

| Priority | Feature | Description | Player Benefit | Dev Cost |
|---|---|---|---|---|
| High | Kitchen tension | Dish spoilage/freshness so cooking isn't fire-and-forget | Turns a chore into a skill | Small |
| High | Trait counterplay | Make Fox/Monkey/etc. things the player responds to, with hints | Guests become characters, not random punishment | Medium |
| High | Art & tone pass | Café-hiding-a-butchery visual identity; make the room feel alive | The USP becomes visible; screenshots sell the game | Large |
| High | Aspirational clientele ladder | Show upcoming guest tiers as a goal board | Long-term pull toward the next unlock | Small |

## Nice To Have

| Priority | Feature | Description | Player Benefit | Dev Cost |
|---|---|---|---|---|
| Med | Audio pass | Cozy ambience + a wrong-note sting on processing | Reinforces the tonal twist | Medium |
| Med | Combo/chain juice | Visible combo meter and streak rewards | Makes flow feel good | Small |
| Med | Named regulars | Persistent guests you recognize before processing them | Emotional weight (and darkness) | Small |
| Low | Daily/shift framing | Optional soft session structure with an end-of-day summary | Natural stopping points & goals | Medium |

## Avoid / Do Not Add

| Feature | Why avoid |
|---|---|
| Multiplayer | No design justification; enormous cost; dilutes a personal, tonal single-player experience |
| Full idle automation | Automating cooking removes the one active skill that differentiates it from a spreadsheet incremental |
| More currencies / systems | The economy is already muddled; add legibility, not more resources |
| A large branching story | The tone is the story; a heavy narrative would bloat scope. Keep it environmental/emergent |
| Twitch/hardcore difficulty | Wrong audience; the appeal is cozy-dark, not stressful |

---

# 6. Missing Gameplay Elements

## Onboarding / Tutorial
**Why expected:** The meta loop is non-obvious; players report not understanding it. **Needed?** Yes — critical. **Implementation:** Scripted first guest that walks the player from cook→serve→fatten→process. **Priority: Critical.**

## Tension / Failure Pressure
**Why expected:** Management games need a squeeze (patience loss, spoilage, capacity). Currently dishes never spoil and losing a guest costs little. **Needed?** Yes — the game is too frictionless to be engaging. **Implementation:** Dish spoilage + meaningful penalty for lost guests (lost meat progress, not just a message). **Priority: High.**

## Feedback / Juice / Audio
**Why expected:** Players expect games to react. There is no audio and near-zero visual feedback. **Needed?** Yes. **Implementation:** SFX, floating numbers, screen reactions to processing. **Priority: High.**

## Visible Guest State
**Why expected:** You can't play around satisfaction/patience/readiness you can't see. **Needed?** Yes. **Implementation:** Per-guest bars for satisfaction, patience, and fattening progress. **Priority: High.**

## Session Structure / Goals
**Why expected:** Endless loops benefit from stopping points and near-term targets. **Needed?** Optional. **Implementation:** Soft "day" framing or milestone goals ("process 3 pigs to unlock…"). **Priority: Medium — the meat/tier ladder can partly serve this.**

## Deliberately NOT missing
Combat, exploration, story branching, crafting complexity — none belong here. The game does **not** need them, and adding them would fight the focused loop.

---

# 7. Content & Replayability Analysis

**How it currently creates reasons to keep playing:**
- **Progression:** meat-gated clientele ladder (13 types), recipe unlocks, 9 upgrades, 11 achievements, prestige multipliers.
- **Randomness:** order rolls, returning-guest chance, trait procs.
- **Choice:** which clientele to attract, which recipes/upgrades to buy, which guest to process.

**Assessment:** On paper the replayability scaffolding is solid for a game this size. In practice it is **gated behind pacing so slow that most players will never see the variety**. Emergent gameplay is minimal — traits fire randomly rather than creating player-driven situations. Different "strategies" barely diverge because early options are so constrained (2 tables, 4 dishes).

**Improvements (in priority order):**
1. **Unlock variety earlier** — front-load the first few clientele/recipe unlocks so the content players already built is actually seen.
2. **Make choices diverge** — e.g. specialize your café toward certain guest types with trade-offs, so two playthroughs feel different.
3. **Turn traits into emergent situations** — with counterplay, a room of Foxes and Monkeys becomes a puzzle, not noise.
4. **Add near-term goals** — visible "next unlock" targets to pull the player forward between the distant prestige walls.

---

# 8. Player Experience Review

## First 10 Minutes
**What the player understands today:** "I click a station to cook, click the dish to carry, click the guest to serve. They eat and leave. I earn some numbers." **What they miss:** why guests return, what fattening is, what meat is for, what the Last Meal Lounge is, and that *the whole game is a fattening-and-processing loop*. The dark hook — the reason to care — never surfaces. **Fix:** a guided first guest that ends on a dramatized first processing within ~5 minutes.

## First Hour
**Does it hook?** Not reliably. Once the cook/serve rhythm is understood, the first hour is repetition toward a distant first process and unlock, with no tension and no tonal payoff. A player who doesn't stumble onto the fattening mechanic will quit confused (as the owner's own playtest nearly did). **The hook is real but currently arrives too late and too quietly.** Fix pacing + dramatize processing and the first hour becomes a slow reveal of a genuinely intriguing premise.

## Long-Term
**What keeps players engaged:** climbing the clientele tiers, chasing recipes/achievements, and prestige loops — *if* they get there. Long-term is the most structurally complete part of the game and the least at risk, provided the early game stops filtering players out. The ceiling is limited by content depth (13 types, 6 recipes) and by whether the tonal fantasy is ever made compelling enough to *want* to keep butchering.

---

# 9. Development Roadmap

## Phase 1: Make It Fun (and land the hook)
**Goals:** A first-time player reaches a dramatized first processing in ~5 minutes and understands the loop.
**Features:** Retune pacing (`visits_until_ready` down, spawn up, start with 3 tables); guided onboarding; per-guest fattening/satisfaction/patience meters; dramatize the Last Meal Lounge (animation + SFX + visible meat payoff); floating gain numbers; clarify currencies.
**Why first:** Nothing else matters if players don't understand the loop or feel its payoff. Every later system is already built and waiting behind this gate.

## Phase 2: Add Depth
**Goals:** Give the moment-to-moment real decisions and tension.
**Features:** Dish spoilage; trait counterplay + hints; combo/chain juice; front-loaded early unlocks; specialization trade-offs.
**Why second:** Once the loop is legible and rewarding, depth converts curiosity into engagement. Tension is what turns a understood loop into a *fun* one.

## Phase 3: Add Content
**Goals:** Sustain the players Phase 1–2 retain.
**Features:** More recipes/events; named recurring regulars; more trait interactions; mid-game milestone goals between prestige walls; optional day/shift framing.
**Why third:** Content is only worth building once players reliably reach it and enjoy the systems it hangs on.

## Phase 4: Polish
**Goals:** Make it something people share and stream.
**Features:** Full art/tone pass (the café-butchery identity), audio design, ambient dining-room life, screenshot/marketing-quality UI, accessibility, save robustness.
**Why last:** Polish amplifies a fun game; it can't rescue an unclear one. But note the *minimum* processing-moment juice belongs in Phase 1 because it's load-bearing for the hook.

---

# 10. Final Assessment

## Strongest Idea
The **tonal inversion**: a pastel, cozy café that is secretly a fattening-and-butchery operation, with charming animal-girls as the livestock. It's memorable, marketable, and — per the owner — "the mixture of cute and macabre will attract people." The meat-web economy (each tier's meat unlocks the next) is a genuinely elegant supporting spine.

## Biggest Risk
**The hook never becomes felt.** Two failure modes compound: (1) pacing so slow and onboarding so absent that players quit before they understand the loop, and (2) presentation so sterile that even players who *do* understand it never feel the transgressive thrill that is the entire point. Right now the game hides its best idea from its own players.

## Missing Ingredient
**A dramatized, early, legible first "processing."** One well-executed Last Meal Lounge moment — reached in the first few minutes, with a fattening meter building to it and a real animated/audio payoff — would retroactively make everything else make sense and land the hook. It is the single highest-leverage addition.

## Unique Selling Point
There are countless cooking/serving games. Almost none let you *cozily fatten adorable characters over repeat visits and then process them into the ingredients that attract the next course of victims.* That specific, uncomfortable, funny fantasy — executed with care — is why someone would choose this over Overcooked or a generic tycoon. Protect it; make it visible; don't sand off the edges into the "wholesome café" the README pretends it is.

## Recommendation
**Continue development, but redesign pacing and presentation around the hook.**

The hardest part — a complete, working economic loop with real content — already exists. The project is not failing on architecture; it's failing on *communication and feel*. That's a far cheaper problem to fix than building systems from scratch. Reduce nothing about the core concept; instead reallocate effort from adding systems to **making the existing systems fast to reach, legible, and tonally alive.** Do Phase 1, playtest again against the owner's original notes, and reassess — but the bones are worth building on.

---

*Note on documentation: reconcile the game's name (Feast Frenzy vs Feast Frenzy) and decide how honest the public README should be about the premise. The current README's "wholesome café" framing works as deliberate marketing misdirection — but the team should choose that consciously, not by accident.*
