# TODO — Feast Frenzy

Every phase of the 2026-07 roadmap landed. What is left is breadth on the
systems that already exist, not new systems.

## Content breadth

- More customer types and a fifth tier hung on the existing trait engine — 13 types across 4 tiers today, and the meat-web has room above Bear.
- More dining events; `dining_events.json` carries only four (dinner rush, health inspector, incognito critic, generous evening).
- More prestige perks; `prestige_perks.json` carries only the four launch choices.
- Day-goal variety, so the closing ledger's "tomorrow's goal" line stops repeating the same shape.

## Presentation

- Decor and room variants driven by the tone system, so a climbing clientele ladder changes the room and not only its tint.

## Engineering

- Deterministic simulation tests for the day cycle, dining events, and returning-regular loops — these three have the least coverage of the shipped systems.
- Add world-anchored floating numbers to `macroquad-toolkit`: the toolkit has toast notifications but `state/floaters.rs` had to be written locally.
