# shared: mob

Who is on a site level to begin with, from the site's secret seed, and what a player is told of anything on the map. Mobs themselves are kept by the server (`server::entities`).

## Files

- `crates/shared/src/mob.rs`: `Spawn`, `spawns`, `EntityLook`. Tests.

## Public API

- `spawns(seed, index, site_kind, level, stairs) -> Vec<Spawn>`: the level's mobs, each with its `number`, kind, position, appearance and `gear` (kinds and qualities, rolled by `item::roll_drop`).
- `Memory` (`remember`, `forget_old`, `warmth_for`), `Event` (`Gave`, `Trespassed(tick)`), `GIVE_REACH`, `TRESPASS_FORGOTTEN_AFTER` (3 minutes of action ticks): what a mob remembers, as facts about characters, each warming or cooling how it feels about them. Only the latest trespass by each character is kept, and it is forgotten after that time; being hurt, once there is combat, is never forgotten (owner, 2026-11-05).
- `EntityLook { id, kind, appearance, feeling, x, y, worn }`: all a player learns of a mob or another player. Never stats. `worn` is what can be seen of what it wears (`Inventory::shown`), for its picture.

## Depends on

- `shared::creature`, `shared::dungeon`, `shared::id`, `shared::item`, `shared::poi`, `shared::rng`, `shared::sheet`.

## Used by

- `server` (`entities`), `protocol`, client `entities`.

## Notes

- Worked out apart from the level's shape, so changing them moves no walls and needs no generator version. Only the server calls it: the seed is secret.
- Which kinds live where, and how many, are first guesses: a few beggars in towns, animals on farms, mostly kobolds elsewhere, with goblins, orcs, a few chickens, the odd giant and some lizardfolk (kobolds start without boots, as they stand on their toes). A number stays with its mob, even if a place couldn't be found for an earlier one. What each kind may start with (`gear_of`) is too, and a blade is then made for what the mob can lift and suits its size, on the server, or left out if none can be; what is worn is made at the mob's size and put straight on, other things are `Medium` and go in its pockets, and a test checks that whoever can have something can wear or carry it; animals have nothing. Nobody starts within `FROM_STAIRS` (6) cells of stairs, so waiting for a reset can't put one beside the way in.
