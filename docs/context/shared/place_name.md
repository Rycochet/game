# shared: place_name

Places' names: towns', castles' and forts'. Deterministic from the poi's map seed and how high it stands, so only the server can work one out (the seed is secret); it sends the name itself.

## Files

- `crates/shared/src/place_name.rs`: `place_name`, `Altitude`, `LOW_BELOW`, `HIGH_FROM`; `base_name` (a beginning and an ending from the lists given; a few tries for one that reads well: no letter doubled across the join, none three times running, not the same word twice). In `place_name/`: `parts.rs` (towns' word lists: `STARTS`, `ENDINGS` with their bands, each size's `Affix::{Before, After}` lists by `affixes(kind)`), `strongholds.rs` (castles and forts: `kind_word`, and `words`: towns' lists plus `MORE_STARTS` and `MORE_ENDINGS`). Tests in `tests.rs` (towns) and `stronghold_tests.rs`.

## Public API

- `place_name(kind, seed, metres) -> Option<String>`: `None` for kinds with no name of their own (anything but towns, castles and forts); `seed` the poi's map seed (`Poi::seed`), `metres` how high it stands above the sea (`movement::WorldGround::metres`). The same whether the place is ruined.
- `Altitude::{Low, Middle, High}`, `Altitude::of_metres` (below `LOW_BELOW` 150 m low, from `HIGH_FROM` 800 m high).

## Depends on

- `shared::rng` (`Seed::derive("town name")`, `Seed::derive("stronghold name")`), `shared::poi` (`PoiKind`).

## Used by

- `server` (`sites/place_name.rs` `place_name_of`: `LevelInfo::place_name` on every level and `ServerMessage::InPoi`'s `name`; `world_map/place_names.rs`: `WorldChunk::name` for places a character has learned).

## Notes

- Towns (owner's design, 2026-11-10): the size word is optional and part of the name, a prefix ("Little", "Village of") or a suffix ("Hamlet", "Cross"), never both; about half of names have one (`AFFIX_PERCENT`). A word may be on several sizes' lists, so a village can be "Ashford Hamlet": flavour, not a promise of the size.
- Castles and forts (owner's design, 2026-11-11): always "Castle" or "Fort" in front and never anything after ("Castle Ravenhold", "Fort Chipstone"), from a wider choice of words than towns (towns' plus sterner ones: Raven, Iron, Storm...; -hold, -keep, -watch, -spire...). Ruins keep their names ("Ruined" goes in front where shown, by the client).
- Height, not nearness to water, makes endings likelier (owner: coasts move over time, so an inland "-mouth" isn't wrong): an ending is `OWN_BAND_WEIGHT` (6) times likelier in its own band, and possible in every band.
- Each kind of name has its own derived seed, drawing nothing from the site generator's sequence: no site, version or site golden test changed. Changing a list, a constant or the order of draws renames places (golden tests). Towns' derived seed is still "town name", from when this module was `town_name`.
- Names aren't unique: repeats are rare, as in real countries.
