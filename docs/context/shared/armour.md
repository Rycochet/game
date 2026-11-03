# shared: armour

What all armour shares, and the pieces for each body location. Multi-location sets come after, as single items built from those pieces (see `decisions.md`, "Armour: sets of pieces").

## Files

- `crates/shared/src/armour/mod.rs`: `Construction`, `Style`.
- `crates/shared/src/armour/hands.rs`: hand pieces: `HandsPiece`, `HandsForm` and what it rolls (`Fingers`, `Cuff`, `Knuckles`), the allowed-combination grid, `roll`, names and weights; `Hand`, a hand's sizes worked out from the body frame. Tests.

## Public API

- `Construction::{Padded, Leather, Mail, Scale, Lamellar, Laminar, PlatesInCloth, MailAndPlate, Splint, Plate, Mirror}` (`ALL`, `flexible`).
- `Style::{White, Gothic, Milanese, Maximilian, Greenwich, Kastenbrust, Elven, Dwarven}` (`ALL`, `name`, `roll`): one list for every location, shared by a set's pieces.
- `hands::roll(rng, piece, style)`: a `HandsForm` for a `HandsPiece::{Gauntlets, Demi, Padded}` (the item kinds `Gauntlets`, `DemiGauntlets`, `PaddedGloves`); a set passes its style, a piece alone rolls one.
- `HandsForm { construction, fingers, cuff, knuckles, claws, locking, material, cloth, trim, style }`: `allowed(piece)`, `name`, `generic_name`, `specific_name`, `grams`, `grams_at(size)`. `Fingers::{Fingered, Mitten, Open}`, `Cuff::{Wrist, Hourglass, Bell, Pointed, Long}` (`reach(frame)`, `flare`), `Knuckles::{Plain, Ridged, Spiked, Rondel}`.
- `hands::Hand::of(frame, side, digits)`: wrist, tip, length, palm, breadth, thickness, thumb, number of fingers, wrist radius, in millimetres. `Hand::digits(fingers)`: digits per hand from a body's finger count.

## Depends on

- `shared::body`, `shared::creature` (`Size`), `shared::material`, `shared::rng`, `serde`.

## Used by

- `shared::item_form` (`ItemForm::Hands`, `give_form`), `shared::wear` (mittens fit any fingers), client `item_art` (gauntlet models) and `debug_menu` (Item Pictures).

## Notes

- Every piece is built in millimetres on `body::Frame`, in place on the body, so a set and a character's picture can use it unchanged.
- A set's pieces share its construction, material, quality, size and style. A style narrows plate (owner's decision): Kastenbrust is mittens with long cuffs, Gothic pointed cuffs, Maximilian fingered with bells, Milanese mostly mittens, Greenwich bells, Dwarven never long. So the style is rolled first.
- Hands: everything is rolled from what the earlier choices allow (construction, fingers, cuff, knuckles, claws, locking, material, cloth, trim), so nothing is thrown away; `allowed` is that grid, checked by a test. Weights are worked out from area × thickness × density, scaled by the material's density against steel (or leather, linen).
- Cuffs reach up the forearm over the arms' vambraces, 28 mm out from the skin, up to 200 mm from the wrist (agreed with the Arms work, whose pieces stay within 20 mm and end at the wrist).
- Claws and knuckle spikes will give unarmed damage bonuses once combat has them (owner's decision).
- Research: `docs/context/research-armour-hands.md`.
