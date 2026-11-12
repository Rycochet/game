# shared: story

What quests' stories are told about: people and households with their names, pets, named bosses, unnamed places, and which way and how far a place lies from a town. Names are sounds and never translated; everything else is a value (`lang::Selector`) each language words. Plots, beats and the `Story` sent to clients come in Flavour text PR 3 (`story-text.md`).

## Files

- `crates/shared/src/story.rs`: `mod` lines, re-exports; `is_people` (every kind listed: animals have no families), `offset` (the near way round a planet, north −y).
- `story/household.rs`: `Household`, `Person`. `story/family_names.rs`: family names' word lists (human, dwarf, elf, halfling; other peoples use their given names' sounds), `pick`.
- `story/pet.rs`: `PetKind`, `Pet`, pets' name sounds. `story/boss.rs`: `Boss`, beasts' name sounds. `story/epithet.rs`: `Epithet`.
- `story/place.rs`: `PlaceKind` (`of_poi`), `Condition`, `PlaceDescription`.
- `story/direction.rs`: `Direction`. `story/distance.rs`: `DistanceBand` and its limits.
- English words: `lang/en/story.ftl`. Tests: `tests.rs` (names, golden), `where_tests.rs`, `words_tests.rs` (every value worded in every language).

## Public API

- `Household::new(kind, seed)`: `family` (`None` for animals) shared by every `member(index) -> Person`. A member's given name is its people's (`CreatureKind::random_name`; animals get pets' names), its gender `Gender::rolled` like mobs'.
- `Person { kind, gender, given, family }` (serde): `full_name(&lang)` in the language's order (`person-full-name`).
- `Pet::new(seed)` (any kind), `Pet::of_kind(kind, seed)`: `{ kind, gender, name }`. `PetKind::{Dog, Horse, Chicken, Cattle}` (`ALL`, `creature()`: a dog is a `Wolf` for now).
- `Boss::new(kind, seed)`: `{ kind, gender, name, epithet }`, named in its people's sounds or beasts' growls; `full_name(&lang)` ("Skarnak the Grey", `boss-full-name`). `Epithet` (16 values, `ALL`).
- `PlaceKind` (15, `ALL`, `of_poi(PoiKind)`, `can_be_ruined()`: not ruins, wilds or oases), `Condition::{Whole, Ruined}`, `PlaceDescription::new(kind, ruined)`, `describe(&lang)` ("ruined tower", no article: add one with `rules().indefinite`).
- `Direction::between(from, to, width)`: 8 ways or `Here` (same spot), whole numbers only, ties to the north, east, south or west way. `DistanceBand::between(from, to, width)`: `Near`, `Short`, `Day`, `Far` by straight distance within `NEAR_WITHIN` (a chunk), `SHORT_WITHIN` (3 chunks), `DAY_WITHIN` (10 chunks), all *to balance*.
- Facts as selectors (`fact-key` messages): `pet-*`, `epithet-*`, `place-kind-*`, `condition-*`, `direction-*`, `distance-*`.

## Depends on

- `shared::rng` (seeds), `shared::creature` (`CreatureKind`, `random_name`, `three_alike`), `shared::gender`, `shared::lang`, `shared::poi` (`PoiKind`, `CHUNK_SIZE`).

## Used by

- Nothing yet: `shared::story`'s plots (Flavour text PR 3) and the Quest system will.

## Notes

- Seeds: callers pass a seed of the household's, pet's or boss's own (never a poi's map seed, which is secret). Each name and gender draws from its own derived seed ("family name", "member" at the index then "given name" and "gender", "pet kind", "pet name", "boss name", "epithet"), so nothing else's golden values changed. Changing a list or a purpose renames people (golden test).
- Given names aren't gendered: peoples' name sounds have no male or female forms yet.
- Positions are world map points; the world wraps east–west, so both take the planet's width.
