# shared: weather

Cloud cover and rain at any place and time, from the public terrain seed and the server's clock, so every client draws the same sky. Purely visual for now.

## Files

- `crates/shared/src/weather.rs`: `WeatherField`, `WeatherSample`, private `Simplex4`. Golden, range, wrap and variety tests.
- `crates/shared/src/weather/lightning.rs`: `WeatherField::lightning`, `LIGHTNING_RAIN`, `LIGHTNING_WARMEST`, `LIGHTNING_AREA`. Tests: none below the threshold or in sleet and snow, how often, shared across an area, golden seconds.

## Public API

- `WeatherField::new(&WorldParams)`, `.sample(x, y, hours) -> WeatherSample` (`hours`: fractional game hours since the world began).
- `WeatherSample { cover, rain }`, both `0..=1`.
- `WeatherField::lightning(x, y, sample, temperature, second) -> bool`: whether lightning flashes there in that whole second of the server's clock. Only where `rain` is above `LIGHTNING_RAIN` (0.85) and the temperature (`climate`'s) is at least `LIGHTNING_WARMEST` (0.3, the shaders' sleet line); up to 9% of seconds in the worst of a storm, none at the threshold. Rolled by position hash on `LIGHTNING_AREA` (32-point) squares, so players near each other see the same flashes.

## Depends on

- `shared::rng`, `shared::world`, `libm`.

## Used by

- Client `common` (`CurrentWorld::weather`), `world_map::sky`, `dungeon_map::rain`; server `ServerWorld::lightning_at`.

## Notes

- `Simplex4` is hand-written, since `fastnoise-lite` stops at 3D and the east-west wrap already uses three dimensions (time is the fourth). Only `+ - * /` and `floor`, seeded by shuffling its table.
- Single clouds on top of larger, slower fronts, with the whole sky drifting east (`WIND`).
- Rain or snow isn't decided here: the client combines `rain` with `shared::climate`'s temperature.
- Takes the world's clock, a float that only counts up (`ServerClock` on the client).
