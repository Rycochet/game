# shared: mob

Who is on a site level to begin with, from the site's secret seed, and what a player is told of anything on the map. Mobs themselves are kept by the server (`server::entities`).

## Files

- `crates/shared/src/mob.rs`: `Spawn`, `spawns`, `EntityLook`. Tests. In `mob/`: `scent.rs` (how strong a trail is, how long a nose follows it), `wild.rs` (the wilderness's mobs; tests in `wild_tests.rs`), `trail.rs` (where they have wandered lately; tests in `trail_tests.rs`).

## Public API

- `spawns(seed, index, site_kind, level, stairs) -> Vec<Spawn>`: the level's mobs, each with its `number`, kind, position, appearance, `attributes` (a build rolled near the centre, `sheet::Build::rolled`, plus its kind's bonus) and `gear` (kinds and qualities, rolled by `item::roll_drop`).
- `wild_spawns(seed, block, level, biome) -> Vec<Spawn>`: a wilderness block's creatures from the secret seed and its position alone: nothing in 45% of blocks (and always in desert), else a pack or herd of 1 to 3 of one kind round a den on open ground (`Ground` or `Grass`): wolves in forests, tundra and taiga; horses and cattle (with a few wolves) in grassland and savanna; a mix in woodland. Each number is a hash, its own across the wilderness. `wild_mob_id(seed, number)`: their ids (UUID v5 of a hash), always the same, so nothing needs saving for one nobody has touched.
- `wild_trail(seed, number, home, level) -> Vec<cell>`: where a wild mob has wandered lately, `home` first: `TRAIL_SHORTEST` (24) to `TRAIL_LONGEST` (96) legal steps out (`movement::step_on_level`), keeping a heading that turns an eighth one step in four, going round what is in the way, stopping at the level's edge or where it can't go on.
- `Memory` (`remember`, `forget_old`, `warmth_for`), `Event` (`Gave`, `Trespassed(tick)`), `GIVE_REACH`, `TRESPASS_FORGOTTEN_AFTER` (3 minutes of action ticks): what a mob remembers, as facts about characters, each warming or cooling how it feels about them. Only the latest trespass by each character is kept, and it is forgotten after that time; being hurt, once there is combat, is never forgotten (owner, 2026-11-05).
- Scent: `scent_strength(inventory)` (percent: `BASE_SCENT` 100 carrying nothing, plus 4 per kg worn and 1 per kg carried, up to `MOST_SCENT` 200; weight stands in for metal), `smells_for(acuity)` (action ticks on the weakest trail: poor 30 s, normal 2 min, keen 5 min, none 0), `can_smell(acuity, strength, laid, now)` (the nose's time × strength), `SCENT_KEPT_FOR` (keenest × strongest: 10 min; older marks can go).
- `EntityLook { id, kind, appearance, feeling, x, y, worn }`: all a player learns of a mob or another player. Never stats. `worn` is what can be seen of what it wears (`Inventory::shown`), for its picture.

## Depends on

- `shared::creature`, `shared::dungeon`, `shared::id`, `shared::item`, `shared::poi`, `shared::rng`, `shared::sheet`.

## Used by

- `server` (`entities`), `protocol`, client `entities`.

## Notes

- Worked out apart from the level's shape, so changing them moves no walls and needs no generator version. Only the server calls it: the seed is secret.
- Which kinds live where, and how many, are first guesses: a few beggars in towns, animals on farms (chickens, horses and cattle), mostly kobolds elsewhere, with goblins, orcs, a few chickens and wolves, the odd giant and some lizardfolk (kobolds start without boots, as they stand on their toes). A number stays with its mob, even if a place couldn't be found for an earlier one. What each kind may start with (`gear_of`) is too, and a blade is then made for what the mob can lift and suits its size, on the server, or left out if none can be; what is worn is made at the mob's size and put straight on, other things are `Medium` and go in its pockets, and a test checks that whoever can have something can wear or carry it; animals have nothing. Nobody starts within `FROM_STAIRS` (6) cells of stairs, so waiting for a reset can't put one beside the way in.
