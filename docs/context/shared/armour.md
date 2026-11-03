# shared: armour

What all armour shares, and the pieces for each body location. Multi-location sets come after, as single items built from those pieces (see `decisions.md`, "Armour: sets of pieces").

## Files

- `crates/shared/src/armour/mod.rs`: `Construction`, `Style`.
- `crates/shared/src/armour/hands.rs`: hand pieces: `HandsPiece`, `HandsForm` and what it rolls (`Fingers`, `Cuff`, `Knuckles`), the allowed-combination grid, `roll`, names and weights; `Hand`, a hand's sizes worked out from the body frame. Tests.
- `crates/shared/src/armour/shoulders.rs`: shoulder pieces, as pairs: `ShouldersKind`, `ShouldersPiece` (what each kind rolls), `ShouldersForm`, `Heraldry`, the grid, `roll`, names, cells and weights. Tests.

## Public API

- `Construction::{Padded, Leather, Mail, Scale, Lamellar, Laminar, PlatesInCloth, MailAndPlate, Splint, Plate, Mirror}` (`ALL`, `flexible`).
- `Style::{White, Gothic, Milanese, Maximilian, Greenwich, Kastenbrust, Elven, Dwarven, Japanese, Chinese, Steppe, Classical, Byzantine, IndoPersian}` (`ALL`, `name`, `roll`, `european`, `allows(construction)`): one list for every location, shared by a set's pieces; saved by name, new ones at the end. `in_style(list, style)`: the constructions in a list a style allows (the whole list if none).
- `hands::roll(rng, piece, style)`: a `HandsForm` for a `HandsPiece::{Gauntlets, Demi, Padded}` (the item kinds `Gauntlets`, `DemiGauntlets`, `PaddedGloves`); a set passes its style, a piece alone rolls one.
- `HandsForm { construction, fingers, cuff, knuckles, claws, locking, material, cloth, trim, style }`: `allowed(piece)`, `name`, `generic_name`, `specific_name`, `grams`, `grams_at(size)`. `Fingers::{Fingered, Mitten, Open}`, `Cuff::{Wrist, Hourglass, Bell, Pointed, Long}` (`reach(frame)`, `flare`), `Knuckles::{Plain, Ridged, Spiked, Rondel}`.
- `shoulders::roll(rng, kind, style)`: a `ShouldersForm` of a `ShouldersKind::{Pauldrons, Spaulders, Guards, Cape, Discs, Ailettes, Wing}`, or any kind (item kind `ShoulderArmour`).
- `ShouldersForm { piece, construction, adornment, finish, material, cloth, trim, style }`: `kind`, `allowed`, `name`, `generic_name`, `specific_name`, `shape` (cells), `grams`, `grams_at(size)`, `roman`, `dendra`, `japanese`. `ShouldersPiece::{Pauldrons { cut, wings, lames, upper_lames, haute_piece, gardbrace, besagews }, Spaulders { lames, besagews, pteruges }, Guards { size, shape, rows, lacing }, Cape { cut, length }, Discs { diameter, boss }, Ailettes { shape, size, heraldry }, Wing}`. `Cut`, `Wings`, `HautePiece` (`height`), `Span` (`guard`, `ailette`), `GuardShape`, `Lacing`, `CapeCut`, `Length`, `AiletteShape`, `Adornment::{None, Spikes(n), Horns, BeastHead, Skull}`, `Finish::{Plain, Tooled, Studded, RingMail}`.
- `Heraldry { field, charge, colour }` (`allowed`, `roll`), `Tincture` (`metal`), `Charge`: a simple coat of arms, for ailettes now and shields and surcoats later.
- `hands::Hand::of(frame, side, digits)`: wrist, tip, length, palm, breadth, thickness, thumb, number of fingers, wrist radius, in millimetres. `Hand::digits(fingers)`: digits per hand from a body's finger count.

## Depends on

- `shared::body`, `shared::creature` (`Size`), `shared::material`, `shared::rng`, `serde`.

## Used by

- `shared::item_form` (`ItemForm::Hands`, `ItemForm::Shoulders`, `give_form`), `shared::item` (a shoulder form's cells and weight), `shared::wear` (mittens fit any fingers), client `item_art` (gauntlet and shoulder models) and `debug_menu` (Item Pictures).

## Notes

- Every piece is built in millimetres on `body::Frame`, in place on the body, so a set and a character's picture can use it unchanged.
- A set's pieces share its construction, material, quality, size and style. A style narrows plate (owner's decision): Kastenbrust is mittens with long cuffs, Gothic pointed cuffs, Maximilian fingered with bells, Milanese mostly mittens, Greenwich bells, Dwarven never long. Each construction is made only in some styles (owner's decision; `Style::allows`): European harnesses in leather, mail, scale, plates in cloth, splint and plate; Japanese without scale or mirrors; Chinese, Steppe, Classical, Byzantine and Indo-Persian in their own; Elven and Dwarven in anything but mirrors; padding in any. So the style is rolled first, then a construction it allows.
- Weight grows with the square of size (owner's decision): area grows, thickness stays (`grams_at`).
- Hands: everything is rolled from what the earlier choices allow (construction, fingers, cuff, knuckles, claws, locking, material, cloth, trim), so nothing is thrown away; `allowed` is that grid, checked by a test. Weights are worked out from area × thickness × density, scaled by the material's density against steel (or leather, linen).
- Cuffs reach up the forearm over the arms' vambraces, 28 mm out from the skin, up to 200 mm from the wrist (agreed with the Arms work, whose pieces stay within 20 mm and end at the wrist).
- Claws and knuckle spikes will give unarmed damage bonuses once combat has them (owner's decision).
- Shoulders: one item is a pair, and a pair may be uneven (Milanese great left, lance cut on the right). Wings and sode reach over chest, back and upper arm but count only as Shoulders. Besagews and gardbraces are parts, not items. Classical plate spaulders are bronze and Mycenaean (Dendra), Classical laminar ones Roman; Japanese guards are sode, named by size; yokes and doublings are Classical. A style leans the kind: ailettes only European or fantasy, the wing only Steppe, guards more in regional styles. Adornments sit on top of any style. Edges agreed with Torso, Arms and Neck: rigid pieces lie outside the breastplate and gorget, lames end about 130 mm down the arm over the rerebrace.
- Research: `docs/context/research-armour-hands.md`, `docs/context/research-armour-shoulders.md`.
