# shared: combat

Combat's rules, the same for players and mobs on both sides: what an attack costs in energy (and how much of it is the wind-up), whether it reaches, and how much harm one that lands does. Not who attacks whom or when (the server, next). Design: `docs/context/abilities-skills.md`, Combat.

## Files

- `crates/shared/src/combat/mod.rs`: re-exports.
- `combat/weapon.rs`: `Weapon { family, grams, length_mm, quality, metal_tier }`, `UNARMED`, `of(item)` (blades only), `handling_grams`, `reach`, `is_edged`.
- `combat/cost.rs`: `attack_cost`, `weapon_cost`, `agility_quickens`, `skill_percent`, `wind_up`, and their constants.
- `combat/damage.rs`: the three stages (`blow`, `past_defence`, `through_armour`), `Blow` (all of them, `harm() -> Points`), `roll_blow`, `off_hand_share`, `quality_factor`, `metal_factor`, `weapon_base`, `strength_hits`.
- `combat/power.rs`: `power`, `SKILL_FLOOR`.
- `combat/wielded.rs`: `Wielded`.
- `combat/fighter.rs`: `Fighter`, `Swing`, `PRACTICE_PER_BLOW`.
- `combat/reach.rs`: `Body`, `ARM_PER_QUARTER`, `MM_PER_UNIT`, `arm_reach`, `reach_between`, `distance`, `within`, `in_reach`.
- `combat/kick.rs`: Kick (`Fighter::kick`, `kick_harm`, `kick_force`, `kick_skill`) and its constants.

## Public API

- `attack_cost(weapon, agility, skill %, dual %?)`: `UNARMED_COST` (2100: two a second at normal speed) plus handling weight × `COST_PER_GRAM_MILLI` (1.75), the weight part quickened by agility (`agility_quickens`, median 100, between half and double; bare hands' part never changes), then up to `SKILL_SAVING` (15%) off by skill, and with a weapon in each hand up to `DUAL_SAVING` (20%) more by dual wielding. `wind_up(cost)`: `ATTACK_WIND_UP` (a quarter) waited before it lands.
- `Weapon::handling_grams`: grams counted for more the longer it is (a metre counts as it weighs), as polearms are slow by leverage. `reach`: how much further than the arm it reaches, in units: its length at `MM_PER_UNIT` (8 mm a unit, a cell about 1.9 m as models are drawn), so spears and polearms reach furthest (free positions PR 7; it was 2 cells for them, 1 for the rest).
- Damage, per mille, to `Points`: `blow` = `UNARMED_DAMAGE` (1 Health) × `weapon_base` (its `weapon_cost` against a fist's, so weight trades speed for size of blow at about the same harm a second) × `strength_hits` (√ of strength against the median) and, for weapons, × `EDGE` (1.2) × `quality_factor` (Poor 0.8 to Artifact 2.0) × `metal_factor` (tier 1 to 4: 0.85, 1.0, 1.1, 1.25) × luck (`roll_blow`: two rolls, 500 to 1500). Then `off_hand_share` (half, plus half the dual skill's percentage), `past_defence` (attack ÷ (attack + defence)) and `through_armour` (`ARMOUR_HALF` 100 ÷ (100 + armour)).
- `power(attributes, leans, total)`: the average of the attributes a skill leans on × its total along its path (`Skills::total`, hundredths), never below `SKILL_FLOOR`. `skill_percent(value)`: a node's own value as a percentage, at most 100.
- `Wielded { main, off }` (`of(inventory)`, `is_dual`, `for_attack(n)`): the weapons held (`creature::Location::Held`), the bigger to swing always the main hand (owner); bare hands when nothing is held; hands take turns when dual wielding, the main first.
- `Fighter { kind, attributes, skills, wielded }` (skills `None` for mobs, counting from the floor): `swing(n)` (the `n`th attack: weapon, hand, cost, wind-up), `attack_power`, `defence_power` (by the main hand's Defence node), `harm(swing, defender, rng)` (armour 0 for now), `practised_by(swing)` (the weapon's node, and `combat/melee/dual` with two weapons), `practised_defending()`. `PRACTICE_PER_BLOW` (8: halved each level up, so all four levels gain; a first guess; the server gives it at most once per skill every 5 seconds, `skill::Windows`). `CANCEL_PENALTY` (420: two ticks at normal speed).
- `Weapon::skill` (its own node by name, `combat/melee/<type>/<weapon>`), `attack_skill(kind)` (that, its type's, or `unarmed/<kind>`), `defence_skill()` (`defence/<type>` or `defence/unarmed`).
- Kick (owner, 2026-11-09; anyone with legs): `Fighter::kick()` an unarmed attack's cost by agility and the Kick skill (`combat/melee/kick`) × `KICK_COST_PER_MILLE` (2500: more than twice); `kick_harm(defender, shoved, rng)` an unarmed blow by the Kick skill's power × `KICK_HARM_PER_MILLE` (400) if it shoves the target (half a cell, free positions PR 5), twice that if it can't (still less than a punch); `kick_force(rng)` against a thing (a door), the blow with no defence × `KICK_FORCE_PER_MILLE` (2000).
- Reach by distance (free positions PR 7): `Body { at: Spot, height, size }` (`Body::on(relief, at, size)`: the ground's height from a `relief::Relief`, 0 where flat). `reach_between(attacker, weapon, target)`: the attacker's radius + `arm_reach` (`ARM_PER_QUARTER` 45 units a quarter of size: 180 for Medium, so bare hands just reach a Medium neighbour diagonally, 339 units) + the weapon's `reach` + the target's radius. `distance` between middles in 3D; `within(level, from, to, range)`: that distance at most `range` and a clear line (`dungeon::spots_are_clear`); `in_reach(level, attacker, weapon, target)`. Heal's range uses `within` (`magic`). For Medium against Medium: fists and a dagger reach the next cell (340, 390), a sword 465, a 2 m spear 590 (two cells straight, not two diagonally).

## Depends on

- `shared::blade` (`Family`), `skill` (`SkillId`, `Skills`, `key_of`), `creature` (`Location::Held`), `item` (`Item`, `Quality`), `item_form` (`ItemForm::Blade`), `material` (`tier`), `sheet` (`Attributes`), `dungeon` (`Level`, `spots_are_clear`), `relief` (`Relief`), `spot` (`Spot`, `root`), `creature` (`Size`), `rng`, `vitals` (`Points`).

## Used by

- `magic` (`Body`, `within`, `skill_percent`), `xtask` (`fight`), `server` (`combat.rs`, `connections/combat.rs`, `entities/fight.rs`), client `abilities` (reach, for automatic use). Bodies come from the server's `Population::relief` and the client's `DrawnGround`.

## Notes

- Integers only, factors per mille, so both sides agree; luck is rolled by the server alone, so a client predicts timings, never results.
- Quality counts most, then metal, then weight (owner, 2026-11-07); all numbers are first guesses to balance by playing. `cargo xtask fight` prints how long fights between equals last with each weapon (about 20 s with fists, 15 to 16 s with weapons, 13 s for a master of two).
- No hit or miss roll: every blow that lands does something; misses are to come from Parry and Dodge. Armour is 0 until armour pieces are given figures.
