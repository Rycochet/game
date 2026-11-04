# shared: footwear

Shoes, boots, sandals, clogs and slippers, all worn in the Middle layer on the feet: what a pair may be like, rolled once and saved, with its names, cells and weight. Not foot armour, which goes over it (`armour::feet`), nor the rules for wearing (`wear`).

## Files

- `crates/shared/src/footwear/mod.rs`: `FootwearForm` (`boot`, `open`), the module list and re-exports.
- `crates/shared/src/footwear/parts.rs`: `FootwearPiece` (one for each item kind, `of`), `Build`, `Height` (`shaft`), `Toe`, `Heel` (`millimetres`), `LONG_POINT`, `BROAD`.
- `crates/shared/src/footwear/details.rs`: `Fastening` (`laced`), `Top`, `Decoration`, `Under`.
- `crates/shared/src/footwear/tables.rs`: builds by kind, heights, toes, heels, backless; `has`.
- `crates/shared/src/footwear/holding.rs`: fastenings by kind, build, height and toe.
- `crates/shared/src/footwear/finish.rs`: top edges, decorations, what is under the sole, dyes, second colours.
- `crates/shared/src/footwear/materials.rs`: uppers, and the sole that follows from them.
- `crates/shared/src/footwear/roll.rs`: `roll`.
- `crates/shared/src/footwear/grid.rs`: `allowed`.
- `crates/shared/src/footwear/names.rs`: specific names by signature, generic names, `name`.
- `crates/shared/src/footwear/weight.rs`: `shape` (cells), `grams`, `grams_at`.

## Public API

- `roll(rng, piece, quality)`: a `FootwearForm` for a `FootwearPiece::{Shoes, Boots, Sandals, Clogs, Slippers}` (the item kinds of the same names; `FootwearPiece::of(kind)`).
- `FootwearForm { piece, build, height, shaft, toe, point, width, heel, heel_mm, backless, fastening, top, decoration, under, upper, sole, dye, contrast }`: `allowed`, `name` (a `String`), `specific_name`, `generic_name`, `shape`, `grams`, `grams_at(size)`, `boot`, `open`.
- `Build::{Sewn, Soft, Rawhide, Woven, Felted, Strapped, Wooden}`, `Height::{Low, Ankle, Calf, Knee, Thigh}` (`shaft`: hundredths of the foot's length), `Toe::{Round, Almond, Pointed, LongPoint, Square, Broad, Upturned, Open}`, `Heel::{Flat, Low, Riding, Louis, Teeth}`, `Fastening`, `Top`, `Decoration`, `Under`, each with `ALL`.

## Depends on

- `shared::armour` (`Dye`), `shared::creature` (`Size`), `shared::item`, `shared::material`, `shared::rng`, `serde`.

## Used by

- `shared::item_form` (`ItemForm::Footwear`, rolled by `give_form`), client `item_art` (models) and `debug_menu` (Item Pictures, Footwear page).

## Notes

- Every choice is rolled from what the earlier ones allow: kind, build, height, toe, heel, fastening, top, upper, decoration, under, dye, second colour. `allowed` is that grid, checked by a test, as are every name turning up and cells by height.
- Owner's decisions: the cavalier boot is the only one over the knee; no platforms (chopine, okobo), no Wellington name, padukas plain wood, nothing on the Outer layer (spurs, pattens, spats, crampons left out for now). Cells: low 2 × 1, ankle and calf 2 × 2, knee and thigh 2 × 3.
- A style's name comes from its signature (closed laces on a sewn low shoe are Oxfords); anything that fits none takes the plain name for its kind, build and height, with an unusual upper first ("Silk Slippers").
- Lengths are in hundredths of the foot's length, so they scale with the frame. Weights are the research's per-pair estimates, changed by the upper and what is added, and by the square of the size, as foot armour's.
- Jewels come only at Rare and better, as gilt finishes do: the grid allows them at any quality.
- Research: `docs/context/research-footwear.md`.
