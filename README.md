# Feast Frenzy

Feast Frenzy is a restaurant simulation game about preparing house recipes, serving hungry guests, and keeping the dining room moving.

You juggle stations, guest demands, recipe timing, and score thresholds while building momentum toward prestige.

## Gameplay

- Start dishes at the right station.
- Serve customers before the restaurant falls behind.
- Clear and redirect stations as demand changes.
- Earn score and rewards from completed service.
- Push toward prestige when the restaurant is ready.

## Goal

Keep guests fed, chain efficient service, and build enough score to prestige into stronger future runs.

## Controls

- Left Click: interact with restaurant UI.
- 1: start Forager Toasts.
- 2: start Hearth Broth.
- 3: start Butcher's Roast.
- 4: start Velvet Sweets.
- C: clear selected station.
- P: prestige if the score threshold is met.

## Current Scope

Playable restaurant loop with named recipes, station control, guest processing, scoring, rewards, and prestige pressure.
# Practical Future Improvements

- Add deterministic tests for kitchen queue order, dining timers, actor states, and growth progression across speed changes.
- Separate actor animation from gameplay state so rendering cannot change restaurant simulation outcomes.
- Move recipe, patience, reward, and upgrade tuning into small fixtures that support quick balance passes.
- Add stress scenarios for peak dining load to catch queue, seating, and order-delivery regressions.

