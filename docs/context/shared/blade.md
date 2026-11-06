# shared: blade

Weapons made of a handle with a blade or head on it, from knives to polearms: what can be made, rolling one, what it is called, weighs and takes up in a grid, and making one to suit a request.

## Files

- `crates/shared/src/blade/mod.rs`: the module list and re-exports, and the helpers the files share. Tests in `tests.rs`. In `blade/`:
  - `family.rs`: `Family`. `parts.rs`: the property enums. `form.rs`: `BladeForm`, its `name`, cells and weight.
  - `cells.rs`: the size grids (`CELLS`), the length spans and which cell a blade falls in.
  - `roll.rs`: `roll_blade`, `roll_blade_at`, `roll_blade_at_within`, `roll_blade_within`, `commission_blade`, `limits_for_wielder`, `Rolled`. `within.rs`: finding lengths and rows within a request's limits. `properties.rs`: `roll_properties`, everything rolled from a row, in three steps in `properties/` (`blade.rs`, `hilt.rs`, `materials.rs`), always in that order: they take rolls in turn from one generator, so reordering them changes every blade.
  - `table.rs`: `Row` (what a name asks for), its shared defaults, and `rows()`, every row in the order they are rolled from. The rows themselves are in `table/`, one file per family (`knives.rs` … `polearms.rs`), in that order: changing the order changes rolls and which of two equal names wins.

## Public API

- `roll_blade(rng, family, quality)`: an ordinary weapon. `roll_blade_at(.., size)`: one that must be this size. `roll_blade_at_within(.., size, limits) -> Option<BladeForm>`: the same within limits, never changing family or size. `roll_blade_within(.., limits) -> Option<Rolled>`: one that suits a request. `commission_blade(rng, name, quality, size, limits)`: a named weapon at any size, keeping its name, only when asked for. `limits_for_wielder(size, spare_grams)`. Items normally get these through `item_form`.
- `BladeForm { family, build, blade, handle, width, edges, curve, point, shape, section, fuller, hands, guard, pommel, ricasso, langets, handle_shape, sides, top, back, tips, blade_metal, handle_material, fittings, decoration, generic_only }`, with `name()`, `attachment()`, `shape()`, `grams()`, `grams_at(size)`. Lengths are whole millimetres at an ordinary person's scale, and an item's size would go on top.
- `weapon_names(family)` (`names.rs`): every name a weapon of the family may go by (its rows', then the generic names of cells its rows reach), for the skill tree.
- `Family::ALL`, and the property enums: `Build`, `BladeWidth`, `Edges`, `Curve`, `Point`, `BladeShape`, `Section`, `Hands`, `Attachment`, `Guard`, `Pommel`, `Ricasso`, `HandleShape`, `SidePart`, `Top`, `Back`, `Tips`, `Decoration`. `Family::kind`, `Rolled { form, size }`.

## Depends on

- `shared::rng`, `shared::item` (`Shape`, `Quality`), `shared::item_form` (`Limits`), `shared::material`, `shared::creature` (`Size`), `serde`.

## Used by

- `item`, `item_form`, `skill`; `server` (`entities`, for mobs' gear); client `item_art` (models) and `debug_menu` (Item Pictures).

## Notes

- Rolling: a row of the family by weight, then a part of a cell its lengths reach, then each property in order within what the row allows. Only cells listed in `CELLS` are ever made, so nothing is rolled and thrown away. Weights are first guesses.
- A blade along the handle and a head at the side use different grids. A side head's "blade" is its cutting edge.
- `name` looks at the form alone: the row asking for the most things wins, then the earlier in the table, or else the cell's generic name. So a weapon rolled from one row may be called by another, which `every_specific_name_turns_up` checks doesn't hide any name. A generic row only exists to be rolled from. A cell with no generic name can only be reached by named rows (Rapier, Nagamaki).
- A blade of 100 cm or more on a grip is always narrow, as a heavy one couldn't be held in one hand.
- `Knife` is the short-blade generic with one edge, `Dagger` with two or a point only.
- A request (owner's rule): an ordinary-sized weapon if any fits, of its family or else another, smaller ones first and in any order, so a tiny creature asking for a sword usually gets something like a dagger. About 10% for smaller wielders and 40% for larger are scaled anyway, nearest the wielder's size, as are those when nothing ordinary fits. Scaled by a roll means `generic_only`: "Giant Sword", never "Giant Katana". A commission keeps its name: "Tiny Katana". Scales everything: weight by size cubed, cells by the size rule.
- Limits never get rolled again: rows that can't fit are passed over, lengths are cut to what fits, and a row too heavy tries its lightest materials before being passed over (keeping `Limits::metal`, the blade metal asked for, which `properties/materials.rs` rolls by weight 0 for the rest: `blade_metals(quality)`). Nothing fitting gives `None`, and the caller decides.
- One cell of length per 20 cm, at most 8; heads with an edge of 15 cm or more are 2 wide. Weight comes from volume and `Material::density`, plus fixed amounts for fittings: first guesses, calibrated to a sword of 1 to 1.5 kg.
- Saved swords from before this have no form and stay as they were.
- Changes from the reviewed tables, to fit the grids: a pilum's head is short, with its long iron shank counted in the handle; a kris stops at 60 cm; a greatsword's handle stops at 40 cm. `Great Axe`, `Short Spear` and `Stabbing Spear` are added as generic names for their cells.
- Where it all came from: `docs/context/research-bladed-weapons.md`.
