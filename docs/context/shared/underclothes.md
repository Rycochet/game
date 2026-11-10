# shared: underclothes

Underclothes, all worn on the Skin layer: shirts and chemises (with stays as an option), braies, breast bands, loincloths, hose, socks, petticoats, linen coifs and wimples. Medieval and earlier first. What one is like is rolled once and saved, with its names, cells and weight. Not the rules for wearing (`wear`), nor clothes over them (the mid-layer, still to come).

## Files

- `crates/shared/src/underclothes/mod.rs`: `UnderForm` (`piece`), the module list and re-exports.
- `piece.rs`: `UnderPiece` (one for each item kind, `of`, `kind`; in `ItemKind::ALL_UNDER`'s order).
- `garment.rs`: `Garment` (which garment and how it is cut: `piece`, `bare`), passing each question to the garment's own file.
- One file per garment, each with its options, roll, grid, names and base weight: `shirt.rs` (`Shirt`, `Length`, `Sleeves`, `Neck`, `Cut`, `Stays`), `braies.rs` (`Braies`, `BraiesLength`, `Fit`, `Waistband`, `Ends`), `band.rs` (`Band`, `BandForm`), `loincloth.rs` (`Loincloth`, `LoinForm`), `hose.rs` (`Hose`, `HoseFeet`, `HoseHeight`, `HoseMake`, `Held`, `Colours`), `socks.rs` (`Socks`, `SockMake`, `Toe`, `SockHeight`, `SockTop`), `petticoat.rs` (`Petticoat`, `SkirtLength`, `SkirtFullness`, `SkirtMake`, `SkirtWaist`), `coif.rs` (`Coif`, `CoifForm`), `wimple.rs` (`Wimple`, `WimpleForm`).
- `cloth.rs`: `Cloth` (plain, lawn, net), `Decoration`; materials, cloths, decorations, dyes and second colours by garment.
- `roll.rs`: `roll`. `grid.rs`: `allowed`. `names.rs`: `specific_name`, `name`. `weight.rs`: `grams`, `grams_at`, `shape`. `table.rs`: `has`, `when`.

## Public API

- `roll(rng, piece, quality)`: an `UnderForm` for an `UnderPiece::{Shirt, Braies, BreastBand, Loincloth, Hose, Socks, Petticoat, Coif, Wimple}` (the item kinds `Shirt`, `Braies`, `BreastBand`, `Loincloth`, `Hose`, `Socks`, `Petticoat`, `LinenCoif`, `Wimple`; `UnderPiece::of(kind)`).
- `UnderForm { garment, material, cloth, decoration, dye, contrast }`: `piece`, `allowed`, `name` (a `String`), `specific_name`, `shape`, `grams`, `grams_at(size)`.
- `Garment::{Shirt(Shirt), Braies(..), Band(..), Loincloth(..), Hose(..), Socks(..), Petticoat(..), Coif(..), Wimple(..)}`; `Garment::bare()`: the kind's locations this one leaves bare (a sleeveless shirt's arms, the feet of footless and stirrup hose), read through `Item::bare` / `Item::worn_locations`.

## Depends on

- `shared::armour` (`Dye`), `shared::creature` (`Location`, `Size`), `shared::item`, `shared::material`, `shared::rng`, `serde`.

## Used by

- `shared::item_form` (`ItemForm::Under`, rolled by `give_form`; material and dye can be asked for), `shared::item` (`Item::bare`), client `item_art` (`item_art/under/`: pictures and the dressed character; `ItemForm::drawn_worn` includes underclothes), `debug_menu` (Item Pictures' Underclothes page).

## Notes

- Every choice is rolled from what the earlier ones allow (garment, material, cloth, decoration, dye, second colour); `allowed` is that grid, checked by a test, as are every name turning up, bare locations and weights.
- Owner's decisions (2026-11-09): one Skin layer; braies, loincloths and petticoats are on the Waist alone (they hang from it), so hose go with them and a biped's Waist takes Skin; stays are an option on long shirts, not a kind; "Linen Coif", and "Hose" never "Chausses", beside the mail armour; footless and stirrup hose leave the feet for socks; nightwear can be worn any time; underclothes stay on at death on Hard. The old `TShirt` kind stays for saves but is no longer made. Drawn worn, they keep close to the body ("tucked in"), and pieces covered at every place they are worn aren't drawn (`Inventory::shown`).
- A name comes from its garment's signature (a twisted strip is a Fundoshi); an unusual material or lawn goes first ("Silk Chemise"), a coif always says its material, stays go after ("Chemise and Stays").
- Gold thread (cauls) only comes at Rare and better; the grid allows it at any quality.
- Weights are the research's estimates from cloth areas (plain linen; woven wool for hose and socks), changed by material and lawn, plus stays, a sole or a codpiece; by the square of the size, as footwear.
- Research: `docs/context/research-underclothing.md`. Players' page: `docs/underclothing.md`, kept in step with this module.
