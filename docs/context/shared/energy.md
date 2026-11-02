# shared: energy

Time for doing things: the same rule for players and, later, mobs, on both sides. A creature gains its speed in energy per tick, can act with 0 or more, and acting spends the action's cost, which can go below 0.

## Files

- `crates/shared/src/energy.rs`: `Energy`, costs. Tests, including against a loop over every tick.

## Public API

- `Energy`: `at`, `ready`, `spend`, `settle` (before the speed changes), `ready_at`.
- `ACTION_TICKS_PER_SECOND` (20), `NORMAL_SPEED` (210 per tick), `SITE_SCALE` (5), `CHANGE_MAP_COST`, `site_step_cost`, `dressing_cost`. World map steps cost `movement::world_step_cost`.

## Depends on

- `shared::movement`, `serde`.

## Used by

- `shared::sheet` (`Sheet::speed`), `server` (`connections`), client `player`, `character`.

## Notes

- Only the energy and the tick it was correct at are stored, and the formula gives any other tick, so nothing loops over creatures and time away only refills them. It stops just under a tick's worth, so waiting saves nothing up and fractions of a tick aren't lost.
- The concept follows a roguelike scheduling article the owner chose, not its queue: this game has no turns to wait for, and the server's intents arrive whenever they do.
- The server's clock is `server::clock::ActionClock`, whole ticks by real time, saved, stopped by pausing and never moved by dev tools. The client counts its own from `Time`, which stops with the game.
- The numbers keep today's pace: 12 cells a second in a site, 60 points on the world map. Whether that is right is a separate question.
- Client: keyboard steps wait for energy here, and travel is paced by distance, which comes to the same, so the server rarely has to wait. Putting clothes on costs energy on the server only, so a following step can wait there a little.
