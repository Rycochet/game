# shared: mob

Who is on a site level to begin with, from the site's secret seed, and what a player is told of anything on the map. Mobs themselves are kept by the server (`server::entities`).

## Files

- `crates/shared/src/mob.rs`: `Spawn`, `spawns`, `EntityLook`. Tests.

## Public API

- `spawns(seed, index, site_kind, level, stairs) -> Vec<Spawn>`: the level's mobs, each with its `number`, kind, position and appearance.
- `Memory` (`remember`, `warmth_for`), `Event`, `GIVE_REACH`: what a mob remembers, as facts about characters, each warming or cooling how it feels about them.
- `EntityLook { id, kind, appearance, feeling, x, y }`: all a player learns of a mob or another player. Never stats.

## Depends on

- `shared::creature`, `shared::dungeon`, `shared::id`, `shared::poi`, `shared::rng`, `shared::sheet`.

## Used by

- `server` (`entities`), `protocol`, client `entities`.

## Notes

- Worked out apart from the level's shape, so changing them moves no walls and needs no generator version. Only the server calls it: the seed is secret.
- Which kinds live where, and how many, are first guesses: a few beggars in towns, animals on farms, mostly kobolds elsewhere. A number stays with its mob, even if a place couldn't be found for an earlier one. Nobody starts within `FROM_STAIRS` (6) cells of stairs, so waiting for a reset can't put one beside the way in.
