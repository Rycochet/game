# shared: creature

Kinds of creature, the same for players and later mobs, and wearing things by the shape of the body.

## Files

- `crates/shared/src/creature.rs`: `Layer`, `Location`, `Size`, `BodyPart`, `CreatureDef`, `CreatureKind` (the table: `def()`). In `creature/`: `layer.rs` (`Layer`, `Location`), `size.rs`, `def.rs` (`BodyPart`, `CreatureDef`), `kind.rs`, `bodies.rs` (each kind's parts: `def()`).
- `crates/shared/src/sheet.rs`: `Stats`, `Skill`, `Ability`, `Trait`, `Appearance` (`suits`, `says`), `Load`, `Sheet` (`new`, `capacity`, `load`). Tests.
- `crates/shared/src/wear.rs`: `Fit`, `WearRefused` (`worth_trying`), `Remover`, what an item covers; on `Made`: `fit`. On `Inventory`, in `wear/`: `check.rs` (`is_worn`, `covered_by`, `check_wear`, what clashes), `changes.rs` (`wear`, `worn_fit`, `change_body`, `may_remove`, `hand_over`), `secrets.rs` (`favour_found_out`, `magic_found_out`, `hide_secrets`). Tests for every rule below.

## Public API

- `CreatureKind::{ALL, def()}`, `CreatureDef { name, weight (grams), size, pockets, body, speed, uses_stairs, playable, stats, skills, abilities, looks }` (`part`, `fingers`). To add a kind: the enum, `ALL` (alphabetical), `def`.
- `BodyPart { location, layers, places }`: which layers a location takes, and how many things taking one place fit there.
- `Layer::{Skin, Middle, Outer, Over}`, `Location`, `Size::{Tiny..Giant}` (`name`, `quarters`, `stepped`, `apart`).

## Depends on

- `serde`, `shared::item`, `shared::id`.

## Used by

- `shared::item`, `protocol`, `server` (`connections`), client `inventory`, `debug_menu`, `dev_tools`.

## Notes

- An item (`item::WearDef`) has one layer and a list of locations (and may be padded, below). It fits if every location is on the body and takes that layer, so a T-shirt needs arms, which a chicken hasn't; trousers need `Legs`, which a horse hasn't.
- Going on and coming off: nothing on a higher layer may be at any of its locations (`covered_by`), and on going on, nothing on the same layer (`WearRefused::Clash`). Taking off is `Inventory::move_item`, which refuses `MoveRefused::Covered`; destroying is refused the same way.
- `one_place` items (rings) take one of a location's places, and clash only once all are taken or something whole is on that layer.
- Padding (`WearDef::padded`): armour with its own padding also takes the Middle layer at those of its locations, for clashing and covering, so Full Armour won't go over a gambeson but goes over trousers. A body must take Middle there. Rules go by `Item::padded`, which is the kind's list, or a torso form's own lining (lined cuirasses, padded jacks, and an armour coat with a lined body), so it can differ between items of one kind.
- `Back` has two places, Outer only, for a backpack and a cloak (both `one_place`). Armour for the back is on `Torso`, so it never stops a pack. Bipeds' `Head` and `Neck` take Skin to Outer (arming caps, coifs) and `Waist` takes Outer (faulds, tassets) as well as Middle (belts).
- `Over` is cloth over armour: a biped's Shoulders, Arms, Torso, Waist and Legs (surcoat, tabard), and a horse's Head, Neck and Torso (caparison over barding). Nowhere else takes it. A piece that only half covers a location doesn't list it (a knee-length surcoat isn't on the legs).
- Rings are Skin on `Fingers`. Gloves and padded gloves are Middle on `Hands` and `Fingers`, gauntlets Outer on both, so both lock rings underneath; fingerless gloves and demi-gauntlets are on `Hands` only, so rings stay free. Rings are always against the skin (owner's decision). Gloves being Middle differs from the owner's first example, so that rings can stay on under them.
- Fit is on the item itself (`item::Made`): the size it fits, the smallest and largest that can wear it, and finger count. `Made::fit` answers `Exact`, `Loose(n)` or `None`; `WearDef::adjustable` things are exact anywhere in their range; magical things are exact within `Quality::magic_reach` sizes of their own (uncommon 1 up to artifact 5), going by their usual range beyond that, whatever the fingers, but not on any body. Things covering whole fingers need exactly the body's count, except mitten gauntlets, which fit any (owner's decision; `wear::made_for_fingers`). Looseness has no penalty until stats have something to affect.
- Taking off: `may_remove(id, Remover)` says who may, by what lies on it: blessed things only their wearer, cursed things nobody, and anything not worn anyone. It is for whoever acts to ask, not `move_item` or `hand_over`: a body falling or a future ability doesn't go by it. God mode skips it for the player's own attempts (`server`).
- Magic gives itself away when it was needed to fit (`magic_found_out`, called only for players). A curse gives itself away when taking it off fails. Nothing reveals a blessing yet. A client doesn't know what is secret, so it lets a player try anything refused only for size or fingers (`worth_trying`), and the server decides.
- A creature's size comes from its kind, apart from its appearance's height, which has no effect on rules.
- The creature is kept on `Inventory` for now, as that is all that depends on it.
- A `Sheet` is one creature's stats, skills and appearance, starting from its kind. Abilities come from the kind alone. Nothing uses stats or skills yet but carrying: capacity is strength × (size + 1) kg, burdened past half and overloaded past all of it, which cut speed (`Sheet::speed`) to 75% and 40%. Appearance never affects rules, and `suits` is how the server checks a choice.
- A creature's body weight comes from its kind, and `Inventory::weight_with_creature` adds what it carries and wears. Load only counts what is carried: a human's body alone is over twice what it can carry.
- `feeling_about` (`sheet::Feeling`) and `reaction` (`sheet::Reaction`) say what a kind thinks of another and does about it: a kind likes its own, kobolds dislike nearly everyone, chickens run from anyone. Memories (`mob::Memory`) shift feelings along a scale (`Feeling::warmth`, `from_warmth`).
- Kinds, bodies, stats and clothes are first guesses to try the rules out.
