# shared: skill

The skill tree as data: every node (Combat › Melee › Longsword › Longsword), what it leans on, and how much one creature has practised each. Practice comes from blows (`combat`), at most once per skill every 5 seconds (`Windows`). Rust takes set-aside skills back towards a floor, by playtime (`rust.rs`). Not yet: checks or specialising (plan: `docs/context/abilities-skills.md`).

## Files

- `crates/shared/src/skill/mod.rs`: re-exports.
- `skill/draft.rs`: `Draft` (a node as written: key, name, leans, children, `Filled`, rust), `branch`, `leaf`, `filled`, `rusting` (a node and what is under it rust this way). `Filled::{Nothing, Weapons(Family), Kinds}`: children made from other tables.
- `skill/branches.rs`: `TOP`, the nine top levels in order; one file per top level in `branches/` (`combat.rs` with `defence.rs` under its melee, `magic.rs`, `athletics.rs`, `stealth.rs`, `survival.rs`, `gathering.rs`, `crafting.rs`, `social.rs`, `lore.rs`). A first draft for the owner to change.
- `skill/tree.rs`: `Tree`, `Node`, `TREE` (a `LazyLock`, built the first time it is needed), `key_of` (a name as a key: lower case, no spaces or hyphens).
- `skill/id.rs`: `SkillId`.
- `skill/practice.rs`: `Skills`, `VALUE_SCALE`, `level_practice`, a whole number cube root.
- `skill/learned.rs`: `Learned` (one node's practice, best, `used` and rust `clock`, in playtime ticks; `gain`, doubled below the best; `floor`; `rust(steps)`), `NOT_SEEN`.
- `skill/rust.rs`: `Rust` (`STEADY`, `DISUSED`), `HOUR`, `STEP_EVERY`, and `Skills::settle`, `used`, `rust`, `rust_for_hours`.
- `skill/opposition.rs`: `LINKS` (one-way, by path), `opposed_by`.
- `skill/saved_node.rs`: one node as read from saves, with or without rust's fields.
- `skill/window.rs`: `Windows`, `WINDOW_TICKS`.
- `skill/familiar.rs`: `familiarity`, `taught`, `FAMILIAR_HALVING`.
- `skill/save_format.rs`: how `Skills` is written in saves and messages.
- `skill/old.rs`: the five skills from before the tree, by name (`old_skill`), and where nodes that moved went (`moved`).

## Public API

- `SkillId`: `all`, `top_levels`, `find(path)`, `name`, `key`, `path`, `parent`, `children`, `depth` (1 at the top), `leans` (its own attributes, or its parent's), `rust` (its own `Rust`, or its parent's), `upwards` (itself, then each node above), `number` / `from_number` (for messages).
- `Skills`: `starting(kind)`, `practice(id)`, `best(id)`, `knows(id)`, `known()` (parents first), `is_empty`, `practise(id, amount)` (flows up, halving each level; doubled below a node's best), `value(id)` (cube root of practice, in hundredths: 100% is 1,000,000), `total(id)` (values along the path: what a check would count).
- Rust (`Skills`): `used(id, played)` (the server, after practising: pushes the clocks of the node and every known node above it to `STEP_EVERY` on, and gives each node it opposes a step if that node's clock has passed; whether anything rusted), `rust(played)` (disuse: a step per `STEP_EVERY` once a node's `disuse_after` has passed since it was used, however often it is asked), `rust_for_hours(hours)` (dying: every known node, clocks untouched), `settle(played)` (on joining: nodes from saves before rust count as used now).
- `Rust { disuse_after, step_per_mille, floor_per_mille }`: `STEADY` (no disuse; a step is 1% of what is above a floor of half the best), `DISUSED` (also after 8 hours unused: Melee). `HOUR`, `STEP_EVERY` (an hour) in action ticks of playtime.
- `key_of(name)`: a name as a node's key (lower case, no spaces or hyphens), for finding a weapon's node by its name (`combat`).
- `VALUE_SCALE` (100), `level_practice(level)` (level³ × 1,000: what an old level, or a kind's starting skill, stands for; level 3 is 30%).
- `Windows<K>` (`take(key, amount, now)`: all of it with no window open, opening one for `WINDOW_TICKS`, 100 action ticks or 5 seconds; inside one only what is above the largest given so far), for the server's one practice per skill every 5 seconds and blocks fought per kind (owner, 2026-11-07).
- `familiarity(fought)` (the square root of blocks fought against a kind, in hundredths like values), `taught(amount, familiarity, value)` (all of it while familiarity is no higher than the skill's own value, then halved every `FAMILIAR_HALVING`, 10%, above it, in a straight line between, rounded): familiar foes teach less, so farming one kind stalls; a test prints the curve of 1,000 kills of one kind (`farming_one_kind_stalls`, `--nocapture`).

## Depends on

- `shared::blade` (`weapon_names`, `Family`), `shared::creature` (`CreatureKind`, each kind's starting skills), `shared::sheet` (`Attribute`), `shared::energy` (`ACTION_TICKS_PER_SECOND`), `serde`.

## Used by

- `shared::sheet` (`Sheet::skills`), `shared::movement` (`Gait`, training), `protocol` (`DevSkills`), `server` (`connections/dev_skills.rs`, `combat.rs`, `heal.rs` and `messages/training.rs` (`used`), `playtime.rs` (`rust` once a minute of playtime), `welcome.rs` (`settle`), `combat/struck.rs` (`rust_for_hours` by `Difficulty::rust_on_death`)), client `character`, `skills_page`.

## Notes

- Under melee, the third level is the type of weapon and the fourth the weapon (`combat/melee/longsword/longsword`, `combat/melee/shortsword/khopesh`); unarmed is `combat/melee/unarmed/<kind>` (owner, 2026-11-06). Weapons are filled in from `blade::weapon_names` (each family's rows and the generic names of cells they reach); kinds from `CreatureKind::ALL`. Club, mace, hammer, flail, staff and whip are written out, as they aren't items yet. Defence is under melee too: `combat/melee/defence/<weapon type>` (no single weapons under it), `defence/unarmed` and the shields (`defence/buckler`, `roundshield`, `kiteshield`, `towershield`), so attack and defence build each other through Melee, and all defending practises Defence for one passive ability (owner, 2026-11-06); a test checks its weapon types match melee's. Shields will be specialisable, the weapon types under Defence not. `combat/melee/dual` (Dual Wielding) stands beside the weapon types, specialisable like them, and has no Defence node (owner, 2026-11-07; the test leaves it out). Ranged has no defence. While shapeshifted, unarmed practice will go to the current form's kind.
- Rust (owner, 2026-11-08): disuse rules are written on a node (`rusting` in the draft) and passed down, like leans, so every weapon under Melee rusts, those filled in from tables too; only Melee has one so far. Opposition is a link (`LINKS`: combat to magic, melee and ranged both ways) that hits only the node it names, never its children, so combat wears down Magic's own share of a spell's check, not each spell; using anything under a link's source counts. A test refuses links within one line of descent (practice flows up, so they could never fire). One clock per node, so disuse and opposition don't stack: at most one step an hour. Rust never takes a node below 1, so known nodes stay known. A test prints the curve (`rust_curve`, `--nocapture`): 50% set aside is 48.6% after a day, 42% after a week; dying leaves 49.3% (Normal) and 48.1% (Hard).
- A node's id is its place in the built tree: client and server agree as they are the same version (`PROTOCOL_VERSION`), and an id out of range is refused when read. Saves keep paths instead (`[("athletics/climbing", 100, 120, 7200, 79200)]` in RON: path, practice, best, used, clock; an older `(path, practice)` reads as used on next joining), so the tree can change; a node that moved is found where it went (`old::moved`: the shields from `combat/shield/`), and a path no longer in it is dropped. A save from before the tree (`{Climbing: 1}`) is read by the old names, each level as `level_practice`.
- A kind's starting skills (`CreatureDef::skills`) are paths and practice; a test checks every path exists.
- Integers only (the cube root is a binary search). Tests check the tree is strict (parents first, paths unique and matching, depth at most 4, everything leans on something), the worked example's numbers, 100% at 1,000,000, windows, and both save formats.
- Saves keep practice, so the change to cube roots (2026-11-07) left old characters' practice as it was and their percentages lower; kinds' starting skills went through `level_practice` and kept theirs.
