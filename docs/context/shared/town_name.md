# shared: town_name

Towns' names: a beginning and an ending that suits how high the town stands, sometimes with one extra word before or after that suits its size. Deterministic from the poi's map seed, so only the server can work one out (the seed is secret); it sends the name itself.

## Files

- `crates/shared/src/town_name.rs`: `town_name`, `Altitude`, `LOW_BELOW`, `HIGH_FROM`; `base_name` (a few tries for one that reads well: no letter doubled across the join, none three times running). In `town_name/`: `parts.rs` (the word lists: `STARTS`, `ENDINGS` with their bands, each size's `Affix::{Before, After}` lists by `affixes(kind)`).

## Public API

- `town_name(kind, seed, metres) -> Option<String>`: `None` if `kind` isn't a town; `seed` the poi's map seed (`Poi::seed`), `metres` how high it stands above the sea (`movement::WorldGround::metres`).
- `Altitude::{Low, Middle, High}`, `Altitude::of_metres` (below `LOW_BELOW` 150 m low, from `HIGH_FROM` 800 m high).

## Depends on

- `shared::rng` (`Seed::derive("town name")`), `shared::poi` (`PoiKind`).

## Used by

- `server` (`sites/place_name.rs`: `LevelInfo::place_name` and `ServerMessage::InPoi`'s `name`; `world_map/town_names.rs`: `WorldChunk::name` for towns a character has learned).

## Notes

- Owner's design (2026-11-10): the size word is optional and part of the name, a prefix ("Little", "Village of") or a suffix ("Hamlet", "Cross"), never both; about half of names have one (`AFFIX_PERCENT`). A word may be on several sizes' lists, so a village can be "Ashford Hamlet": flavour, not a promise of the size.
- Height, not nearness to water, makes endings likelier (owner: coasts move over time, so an inland "-mouth" isn't wrong): an ending is `OWN_BAND_WEIGHT` (6) times likelier in its own band, and possible in every band.
- The name's own derived seed draws nothing from the town generator's sequence: adding names changed no site, version or golden test. Changing a list, a constant or the order of draws renames every town (golden test).
- Names aren't unique: with about 1600 bases and the extra words, repeats are rare, as in real countries.
