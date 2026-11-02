# shared: blade

Weapons made of a handle with a blade or head on it, from knives to polearms: what can be made, rolling one, and what it is called. Not an item yet: nothing calls this outside its tests until `ItemKind` and `ItemForm` take it up.

## Files

- `crates/shared/src/blade/mod.rs`: `BladeForm` and its properties, the size grids (`CELLS`), `roll_blade`, `name`. Tests.
- `crates/shared/src/blade/table.rs`: `ROWS`, the named weapons and what each asks for.

## Public API

- `roll_blade(rng, family) -> BladeForm`.
- `BladeForm { build, blade, handle, width, edges, curve, point, shape, section, fuller, hands, guard, pommel, ricasso, langets, handle_shape, sides, top, back, tips }`, with `name()` and `attachment()`. Lengths are whole millimetres at an ordinary person's scale, and an item's size would go on top.
- `Family::ALL`, and the property enums: `Build`, `BladeWidth`, `Edges`, `Curve`, `Point`, `BladeShape`, `Section`, `Hands`, `Attachment`, `Guard`, `Pommel`, `Ricasso`, `HandleShape`, `SidePart`, `Top`, `Back`, `Tips`.

## Depends on

- `shared::rng`, `serde`.

## Used by

- Nothing yet.

## Notes

- Rolling: a row of the family by weight, then a part of a cell its lengths reach, then each property in order within what the row allows. Only cells listed in `CELLS` are ever made, so nothing is rolled and thrown away. Weights are first guesses.
- A blade along the handle and a head at the side use different grids. A side head's "blade" is its cutting edge.
- `name` looks at the form alone: the row asking for the most things wins, then the earlier in the table, or else the cell's generic name. So a weapon rolled from one row may be called by another, which `every_specific_name_turns_up` checks doesn't hide any name. A generic row only exists to be rolled from. A cell with no generic name can only be reached by named rows (Rapier, Nagamaki).
- A blade of 100 cm or more on a grip is always narrow, as a heavy one couldn't be held in one hand.
- `Knife` is the short-blade generic with one edge, `Dagger` with two or a point only.
- Materials and styling aren't here yet: they come with making these into items, as `Material` has no wood or similar so far.
- Changes from the reviewed tables, to fit the grids: a pilum's head is short, with its long iron shank counted in the handle; a kris stops at 60 cm; a greatsword's handle stops at 40 cm. `Great Axe`, `Short Spear` and `Stabbing Spear` are added as generic names for their cells.
- Where it all came from: `docs/context/research-bladed-weapons.md`.
