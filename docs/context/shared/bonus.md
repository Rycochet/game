# shared: bonus

Bonuses: what worn items and effects (food, potions, dev tools) add to a creature's attributes, the most Health and Mana, skills, and things only bonuses give (restoring Health or Mana over time). Rules only: the server applying them to players (wearing, eating) and the client showing them are elsewhere.

## Files

- `crates/shared/src/bonus.rs`: the module list, re-exports, `MILLI`. Tests.
- `bonus/stat.rs`: `Stat::{Attribute(a), Most(vital), Skill(id), Restore(vital)}` (new ones at the end), `Amount::{Flat, Percent}`, `Bonus { stat, amount }` (`flat`, `percent`), `PER_CENT`.
- `bonus/totals.rs`: `Totals` (`of`, `sum`, `added_milli`, `whole`, `attributes`, `restoring`), `Sum { flat, percent }`, `round_milli`.
- `bonus/item.rs`: `item_bonuses(item)`, `quality_factor`, `TOUGHNESS_PER_KG` 10, `AGILITY_PER_KG` 2, `MANA_PER_KG` 1.
- `bonus/material.rs`: `MaterialFactors { toughness, agility, mana }` (`USUAL`), `material_factors(material)`; an armour piece's material from its form.
- `bonus/effect.rs`: `Effect { source, bonuses, start, end, ends_on }` (`lasts`, `dev_bonus`), `Source::{Eaten(kind), Dev}`, `EndsOn { moving, hurt }` (`NOTHING`, `by`), `Cancel::{Moving, Hurt}`.
- `bonus/effects.rs`: `Effects(Vec<Effect>)` (`add`, `cancel`, `cancelled_by`, `forget_ended`, `first_ended`, `next_end`, `bonuses(now)`).
- `bonus/food.rs`: `food_effect(kind, now)`, `is_food(kind)`, `APPLE_RESTORE`, `APPLE_TICKS`.
- On `Sheet` (`sheet/bonuses.rs`): `Situation { worn, playtime, refill }`, `totals`, `attributes_with`, `limits`, `rebase`, `change_effects`, `add_effect`, `eat`, `cancel_effects`, `forget_ended`; old saves' changes folded into effects by `settled`.

## Public API

- Everything above. Amounts are thousandths of the stat's unit (points for attributes and the most, a skill's value hundredths for skills, points a second for restoring); percentages are thousandths of a percent of the stat's base (what it is without bonuses).
- `Sheet::limits(situation, now)` is what rules go by for Health and Mana; `Sheet::attributes_with(worn, now)` for attributes.

## Depends on

- `sheet` (`Attribute`, `Attributes`), `vitals` (`Vital`, `PER_POINT`), `skill` (`SkillId`), `item`, `item_form`, `material`, `energy` (ticks a second), `serde`.

## Used by

- `sheet` (`Sheet::effects`, `sheet/bonuses.rs`), `vitals` (`Totals` in `Vitals::most` and `limits_with`), `server` (`connections/vitals.rs`, `bonuses.rs`, `consume.rs`, `ground/eat.rs`, `entities/made.rs`), client `character` (the most's lines), `inventory` (`is_food`: eating on the clothes panel).

## Notes

- Totals are always rebuilt from scratch from everything worn plus effects lasting at `now` (`Sheet::totals`), flat sum then percentage of the base, rounded once to the nearest (halves up); never added or taken away a piece at a time, so no rounding builds up. Attributes never go below 1.
- Item bonuses are a pure function of the item. Only `ItemCategory::Armour` gives anything so far (owner, 2026-11-14): per kg of the piece's own weight (its form's, without contents), +10 Toughness × quality factor × material factor, −2 Agility × material factor, −1 most Mana × material factor. Quality factors (first guesses): Poor 0.75, Common 1, +0.25 a step to Legendary 2, Artifact 2.5; no quality counts as Common. Material factors are a table, all 1 for now (later Mithril costing less Agility, Adamantine giving more Toughness).
- Effects: at most one per `Source`, so eating a second apple starts it again. They end at an action tick (real time, offline too) or by `Cancel` (`EndsOn`). `lasts` goes by the end only, so a client passing tick 0 counts everything the server still keeps. An apple: 1 Health a second for 30 s, ended by moving or being hurt (owner, 2026-11-14).
- Restoring joins refilling's formula (`vitals::Rate`): extra points a second while refilling (so not while waiting after a fight), until `Limits::restore.until`, the next tick any effect ends. `Sheet::forget_ended` settles each ended effect at its own end with the limits from just before, so restoring stops exactly there however late it is called. `Sheet::limits` doesn't count an ended effect, so call `forget_ended` first (`change_effects` and `cancel_effects` do); cancel `Hurt` before harming.
- Changes to the most from bonuses keep the share (`Vitals::rebase`, owner, 2026-11-14): `Sheet::change_effects` and `Sheet::rebase` (for worn changes: limits before, then the change) do it.
- The server applies worn items and effects to players (`connections/bonuses.rs`, rebuilt on any change to what is worn) and worn items to mobs once, when they are made; players eat by dropping food on the clothes panel (`ClientMessage::Consume`). `is_food` says what can be eaten.
