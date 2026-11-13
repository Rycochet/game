# shared: quest

Quests generated from a town's seed, the same for everyone: who gives them (town buildings, by slots), what they ask (a step graph on hooks of nearby pois or the wilds round the town), what they need, what they pay, how they move alignment, and their story through `shared::story`. Only the server calls the generator: pois' seeds are secret. Offering, accepting, progress and saves are the server's (`server/server.md`, `quests`). Plan: `quests.md`.

## Files

- `crates/shared/src/quest.rs`: `mod` lines and re-exports.
- Model: `quest/kind.rs` (`QuestKind`), `id.rs` (`QuestId`, `Slot`, `earlier`), `requirement.rs` (`Requirement`, `Need`, `Standing`), `step.rs` (`Step`, `Target`, `Alternative`, `Way`), `hook.rs` (`Hook`, what a poi offers, the kind its kills are of), `model.rs` (`Quest`, open steps).
- Generator: `world.rs` (`QuestWorld`, `radius`, `is_larger`, distances, pois near a poi), `owner.rs` (who owns an exclusive hook), `area.rs` (a town's candidates by weight), `givers.rs` (`Giver`, `givers`, the quest seed, each building's household), `template.rs` (a hook's steps, a second step at the same poi, a poi's boss and unique item), `remains.rs` (`natural_parts(kind)`: the parts a kind has that a quest may ask for, by its body (`body::Parts::of`, `has_hands`): fingernails with hands, toenails on a person's feet, fangs on a muzzle, claws, a tail, horns, tusks, ears, feathers, hooves, scales on kobolds and lizardfolk, hearts and livers on all; `remains_of(poi, kind)`, the poi's own, from its seed), `plan.rs` (a plan's steps and the facts for its story), `reward.rs`, `telling.rs` (the `story::Request`), `make.rs` (a plan into a `Quest`), `slot.rs` (a slot's chain, its needs), `town.rs` (`town_quests`), `feeder.rs` and `feeder_quest.rs` (chains across towns), `daily.rs`.
- Tests: `tests.rs` (the rules over a few dozen towns of the test world), `golden_tests.rs`, `print_tests.rs` (`cargo test -p shared print_quests -- --ignored --nocapture` prints three towns' quests and dailies as a player reads them, with what the server keeps), `test_world.rs`.

## Public API

- `QuestWorld { pois, roads, name_of }`: what the generator reads; `name_of(&Poi) -> Option<String>` is the server's `place_name_of` (towns, castles, forts).
- `town_quests(world, town, buildings) -> Vec<Quest>`: a town's normal quests (`buildings`: its site's `BuildingLook`s): every giver's slots, each a chain, then its feeder. Empty for anything but a town.
- `daily_givers(town, buildings)`, `daily_quest(world, town, buildings, building, day) -> Option<Quest>`, `utc_day(unix_seconds)`, `SECONDS_PER_DAY`.
- `givers(town, buildings) -> Vec<Giver { building, kind, slots, daily }>`.
- `feeder_destination(world, town) -> Option<Poi>` (the larger road neighbour its feeder goes to), `feeds_into(world, town)` (smaller neighbours whose feeders come here), `is_larger(a, b)` (size, then chunk), `radius(kind, daily)` (chunks: hamlet 3, village 4, town 6, city 7; dailies half).
- `owner(world, poi, hook) -> Option<ChunkPos>`: the town whose quests may use an exclusive hook (`None` for shared ones).
- `Quest { id, kind, building, member, requirements, steps, reward, deeds, story, ending }`; `open_steps(done)`, `ready_to_turn_in(done)`, `is_hidden_by_alignment()`. `story` goes with the offer; `ending` stays on the server until turn-in.
- `QuestId { town, slot }`, `Slot::{Giver { building, slot, chain }, Feeder, Daily { building, day }}`, `earlier()`. Serde: the id is all a save or a message keeps; the rest is made again.
- `QuestKind::{Normal, Daily, Artifact}` (Artifact not generated yet: PR 11).
- `Requirement { need, hidden }` (`shown`, `hidden`, `hides_quest`), `Need::{Standing { town, least }, Done(QuestId), Skill { skill, least }, Alignment(Band)}`, `Standing::{Stranger, Known, Trusted, Honoured, Champion}`.
- `Step { goal, target, count, after, alternatives }` (`is_open(done)`), `Target::{Poi { chunk, hook }, Wilds { chunk }, Town { chunk }}`, `Alternative { way, skill }`, `Way::{Carry, Tame, Intimidate}`.
- `Hook::{Boss, Item, Escortee, Kills(kind), Explore, Visit, Gather, Collect(kind)}` (`Collect`: a part of the poi's creatures of its counted kills' kind brought back, worded as a Gather step, "Gather kobold scales 2/5"; shared, weight 2), `is_exclusive`, `of_poi(&Poi)`.
- `boss_of(&Poi) -> story::Boss`: a poi's quest boss (kind, gender, name, epithet) from its own seed: the server spawns it as the story names it. `boss_tier(&Poi) -> Quality`: the boss's tier (owner: a better version of its kind; Claude's weights: Uncommon 50, Rare 30, Epic 15, Legendary 5), from the poi's seed. `item_of(&Poi) -> story::ItemCategory`: its unique item, a keepsake, which the server makes as the story names it.

## Depends on

- `shared::story` (households, bosses, pets, places, `Request`, `tell`), `shared::poi` (`PoiMap::in_area`, `in_chunk`), `shared::road` (`Roads::links_in_area`, `RoadKind` for sizes), `shared::site` (`BuildingLook`, `BuildingKind`, `Generator::for_site` for ruins), `shared::mob` (`who_lives_in`), `shared::alignment` (`Band`, `Deed`), `shared::skill` (`SkillId`), `shared::rng`, `shared::creature`.

## Used by

- The server (`server/server.md`, its `quests` module): offers, taking, handing in, progress, all through `town_quests`, `daily_quest` and the model, and its `entities` for bosses (`owner`, `boss_of`); `shared::protocol` carries `Step`, `QuestId`, `QuestKind`, `Standing` in its quest messages.

## Notes

- Seeds: everything from `poi.seed.derive("quests")` (per giver, slot, chain or day by `at`), drawing nothing from the site's sequences. A poi's quest boss, unique item and kills kind come from that poi's own seed ("quest boss", "quest item", "quest kills"), so every town's quest and the site agree. Story and household seeds are derived, never the poi seed itself. Changing a list, weight or draw order changes every town's quests (golden test).
- Not overlapping (owner's rule): a boss, an item and a lost one (exclusive hooks) belong to one of the towns reaching the poi, by `hash(poi, hook)` (`owner`), and within a town to one quest. Kills, exploring, visiting and gathering are shared; each use within a town halves a hook's chances. Dailies only use shared hooks.
- Givers (Claude's figures, *to balance*): 30% of houses one slot; other buildings one, plus a second by town size (10 to 40%); inns and a quarter of other shops give dailies. Slot `n`'s giver is the building's household's member `n`; townsfolk are human until towns have peoples. Chains are 1 to 3 quests (2 in hamlets): each needs the one before and Known, then Trusted standing, and keeps its reason, giver and relatives (`Story::carry`). 8% of chains outside hamlets need an Evil alignment (hidden entirely) and are told dark; they move no alignment yet (PR 10). 30% of gathering quests have a hidden herbalism need.
- Templates: one step on a hook, sometimes (35%) a second at the same poi: kills before a boss or exploring before a find (in series), or kills alongside a find or exploring (in parallel). Escorts are of a pet (other ways: carry it, or tame it with Animals) or of a relative (Rescued). Gathering is herbs, at a herbalist's hut or in the wilds round the town. Turning in isn't a step: it follows them all.
- Across towns (owner): only smaller road neighbours are read, never their quests. A link counts whether or not water cuts its road. Each town's feeder (a delivery) goes to one larger neighbour by its own hash; that town gives one first quest of a chain (not a dark one) a need for it.
- Rewards: copper by town size, work, distance and chain place, rounded to 5, dailies half; 20% of normal quests add an item, training or a favour (the story then may tell season and weather, which aren't set yet).
- Generating a town's quests takes milliseconds (the tests make a few dozen towns' quests in about 0.15 s); the server should still keep them per town.
