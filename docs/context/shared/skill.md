# shared: skill

The skill tree as data: every node (Combat › Melee › Longsword › Longsword), what it leans on, and how much one creature has practised each. Not yet: practice from use, checks, rust or specialising (plan: `docs/context/abilities-skills.md`).

## Files

- `crates/shared/src/skill/mod.rs`: re-exports.
- `skill/draft.rs`: `Draft` (a node as written: key, name, leans, children, `Filled`), `branch`, `leaf`, `filled`. `Filled::{Nothing, Weapons(Family), Kinds}`: children made from other tables.
- `skill/branches.rs`: `TOP`, the nine top levels in order; one file per top level in `branches/` (`combat.rs` with `defence.rs` under its melee, `magic.rs`, `athletics.rs`, `stealth.rs`, `survival.rs`, `gathering.rs`, `crafting.rs`, `social.rs`, `lore.rs`). A first draft for the owner to change.
- `skill/tree.rs`: `Tree`, `Node`, `TREE` (a `LazyLock`, built the first time it is needed), `key_of` (a name as a key: lower case, no spaces or hyphens).
- `skill/id.rs`: `SkillId`.
- `skill/practice.rs`: `Skills`, `VALUE_SCALE`, `level_practice`.
- `skill/save_format.rs`: how `Skills` is written in saves and messages.
- `skill/old.rs`: the five skills from before the tree, by name (`old_skill`), and where nodes that moved went (`moved`).

## Public API

- `SkillId`: `all`, `top_levels`, `find(path)`, `name`, `key`, `path`, `parent`, `children`, `depth` (1 at the top), `leans` (its own attributes, or its parent's), `upwards` (itself, then each node above), `number` / `from_number` (for messages).
- `Skills`: `starting(kind)`, `practice(id)`, `knows(id)`, `known()` (parents first), `is_empty`, `practise(id, amount)` (flows up, halving each level), `value(id)` (square root of practice, in hundredths), `total(id)` (values along the path: what a check would count).
- `VALUE_SCALE` (100), `level_practice(level)` (level² × 100: what an old level stood for).

## Depends on

- `shared::blade` (`weapon_names`, `Family`), `shared::creature` (`CreatureKind`, each kind's starting skills), `shared::sheet` (`Attribute`), `serde`.

## Used by

- `shared::sheet` (`Sheet::skills`), `protocol` (`DevSkills`), `server` (`connections/dev_skills.rs`), client `character`.

## Notes

- Under melee, the third level is the type of weapon and the fourth the weapon (`combat/melee/longsword/longsword`, `combat/melee/shortsword/khopesh`); unarmed is `combat/melee/unarmed/<kind>` (owner, 2026-11-06). Weapons are filled in from `blade::weapon_names` (each family's rows and the generic names of cells they reach); kinds from `CreatureKind::ALL`. Club, mace, hammer, flail, staff and whip are written out, as they aren't items yet. Defence is under melee too: `combat/melee/defence/<weapon type>` (no single weapons under it), `defence/unarmed` and the shields (`defence/buckler`, `roundshield`, `kiteshield`, `towershield`), so attack and defence build each other through Melee, and all defending practises Defence for one passive ability (owner, 2026-11-06); a test checks its weapon types match melee's. Shields will be specialisable, the weapon types under Defence not. `combat/melee/dual` (Dual Wielding) stands beside the weapon types, specialisable like them, and has no Defence node (owner, 2026-11-07; the test leaves it out). Ranged has no defence. While shapeshifted, unarmed practice will go to the current form's kind.
- A node's id is its place in the built tree: client and server agree as they are the same version (`PROTOCOL_VERSION`), and an id out of range is refused when read. Saves keep paths instead (`[("athletics/climbing", 100)]` in RON), so the tree can change; a node that moved is found where it went (`old::moved`: the shields from `combat/shield/`), and a path no longer in it is dropped. A save from before the tree (`{Climbing: 1}`) is read by the old names, each level as `level_practice`.
- A kind's starting skills (`CreatureDef::skills`) are paths and practice; a test checks every path exists.
- Integers only. Tests check the tree is strict (parents first, paths unique and matching, depth at most 4, everything leans on something), the worked example's numbers, and both save formats.
