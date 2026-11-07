# shared: energy

Time for doing things: the same rule for players and mobs, on both sides. A creature gains its speed in energy per tick, can act with 0 or more, and acting spends the action's cost, which can go below 0.

## Files

- `crates/shared/src/energy.rs`: `Energy`, costs. Tests, including against a loop over every tick.

## Public API

- `Energy`: `at`, `ready`, `spend`, `settle` (before the speed changes), `ready_at`, `cancel(tick, speed, penalty)` (gives back what is owed beyond `penalty`: an ability called off before it lands).
- `ACTION_TICKS_PER_SECOND` (20), `NORMAL_SPEED` (210 per tick), `SITE_SCALE` (5: stairs, dressing, handling), `SITE_STEP_SCALE` (15: steps in a site), `SITE_CELLS_PER_SECOND` (4, what the client paces site travel and others' tokens by), `CHANGE_MAP_COST`, `site_step_cost`, `dressing_cost`, `handling_cost` (moving between containers: 10 kg is one straight step, to depend on stats later). World map steps cost `movement::world_step_cost`, less along roads.

## Depends on

- `shared::movement`, `serde`.

## Used by

- `shared::sheet` (`Sheet::speed`), `server` (`connections`), client `player`, `character`.

## Notes

- Only the energy and the tick it was correct at are stored, and the formula gives any other tick, so nothing loops over creatures and time away only refills them. It stops just under a tick's worth, so waiting saves nothing up and fractions of a tick aren't lost.
- The concept follows a roguelike scheduling article the owner chose, not its queue: this game has no turns to wait for, and the server's intents arrive whenever they do.
- The server's clock is `server::clock::ActionClock`, whole ticks by real time, saved, stopped by pausing and never moved by dev tools. The client counts its own from `Time`, which stops with the game.
- Pace: 4 cells a second in a site (owner, 2026-11-05: 12 let a mob coming round a corner reach you within a second; tweak `SITE_STEP_SCALE`), 60 points on the world map. Only steps were slowed: stairs, dressing and handling still go by `SITE_SCALE`, so they take as long as before.
- Client: keyboard steps wait for energy here, and travel is paced by distance, which comes to the same, so the server rarely has to wait. What is done with items costs the same on both sides: `common::SpentEnergy` spends it here and stops a journey for that long, as travel is paced by distance.
