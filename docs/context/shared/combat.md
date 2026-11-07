# shared: combat

Combat's rules, the same for players and mobs on both sides: what an attack costs in energy (and how much of it is the wind-up), whether it reaches, and how much harm one that lands does. Not who attacks whom or when (the server, next). Design: `docs/context/abilities-skills.md`, Combat.

## Files

- `crates/shared/src/combat/mod.rs`: re-exports.
- `combat/weapon.rs`: `Weapon { family, grams, length_mm, quality, metal_tier }`, `UNARMED`, `of(item)` (blades only), `handling_grams`, `reach`, `is_edged`.
- `combat/cost.rs`: `attack_cost`, `weapon_cost`, `strength_lightens`, `skill_percent`, `wind_up`, and their constants.
- `combat/damage.rs`: the three stages (`blow`, `past_defence`, `through_armour`), `Blow` (all of them, `harm() -> Points`), `roll_blow`, `off_hand_share`, `quality_factor`, `metal_factor`, `weapon_base`, `strength_hits`.
- `combat/power.rs`: `power`, `SKILL_FLOOR`.
- `combat/wielded.rs`: `Wielded`.
- `combat/reach.rs`: `in_reach`.

## Public API

- `attack_cost(weapon, strength, skill %, dual %?)`: `UNARMED_COST` (2100: two a second at normal speed) plus handling weight × `COST_PER_GRAM_MILLI` (1.75), the weight part lightened by strength (`strength_lightens`, median 100, between half and double), then up to `SKILL_SAVING` (15%) off by skill, and with a weapon in each hand up to `DUAL_SAVING` (20%) more by dual wielding. `wind_up(cost)`: `ATTACK_WIND_UP` (a quarter) waited before it lands.
- `Weapon::handling_grams`: grams counted for more the longer it is (a metre counts as it weighs), as polearms are slow by leverage. `reach`: 2 cells for polearms and spears, else 1 (owner).
- Damage, per mille, to `Points`: `blow` = `UNARMED_DAMAGE` (1 Health) × `weapon_base` (its `weapon_cost` against a fist's, so weight trades speed for size of blow at about the same harm a second) × `strength_hits` (√ of strength against the median) and, for weapons, × `EDGE` (1.2) × `quality_factor` (Poor 0.8 to Artifact 2.0) × `metal_factor` (tier 1 to 4: 0.85, 1.0, 1.1, 1.25) × luck (`roll_blow`: two rolls, 500 to 1500). Then `off_hand_share` (half, plus half the dual skill's percentage), `past_defence` (attack ÷ (attack + defence)) and `through_armour` (`ARMOUR_HALF` 100 ÷ (100 + armour)).
- `power(attributes, leans, total)`: the average of the attributes a skill leans on × its total along its path (`Skills::total`, hundredths), never below `SKILL_FLOOR`. `skill_percent(value)`: a node's own value as a percentage, at most 100.
- `Wielded { main, off }` (`of(inventory)`, `is_dual`, `for_attack(n)`): the weapons held (`creature::Location::Held`), the bigger to swing always the main hand (owner); bare hands when nothing is held; hands take turns when dual wielding, the main first.
- `in_reach(level, from, to, reach)`: within `reach` cells (diagonals count one), not the same cell, and a clear line (`dungeon::line_is_clear`).

## Depends on

- `shared::blade` (`Family`), `creature` (`Location::Held`), `item` (`Item`, `Quality`), `item_form` (`ItemForm::Blade`), `material` (`tier`), `sheet` (`Attributes`), `dungeon` (`Level`, `line_is_clear`), `rng`, `vitals` (`Points`).

## Used by

- `xtask` (`fight`). Server and client combat to come.

## Notes

- Integers only, factors per mille, so both sides agree; luck is rolled by the server alone, so a client predicts timings, never results.
- Quality counts most, then metal, then weight (owner, 2026-11-07); all numbers are first guesses to balance by playing. `cargo xtask fight` prints how long fights between equals last with each weapon (about 20 s with fists, 15 to 16 s with weapons, 13 s for a master of two).
- No hit or miss roll: every blow that lands does something; misses are to come from Parry and Dodge. Armour is 0 until armour pieces are given figures.
