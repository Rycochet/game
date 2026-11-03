# shared: armour

What all armour shares. Pieces for each location are added by their own work; multi-location sets come after, as single items built from those pieces (see `decisions.md`, "Armour: sets of pieces").

## Files

- `crates/shared/src/armour.rs`: `Construction`.

## Public API

- `Construction::{Padded, Leather, Mail, Scale, Lamellar, Laminar, PlatesInCloth, MailAndPlate, Splint, Plate, Mirror}` (`ALL`, `flexible`).

## Depends on

- `serde`.

## Used by

- Nothing yet.

## Notes

- Every piece is built in millimetres on `body::Frame`, in place on the body, so a set and a character's picture can use it unchanged.
- A set's pieces share its construction, material, quality and size.
- Research: the armour research files (to go in with the first pieces).
