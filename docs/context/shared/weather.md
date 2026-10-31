# shared: weather

Cloud cover and rain at any place and time, from the public terrain seed and the server's clock, so every client draws the same sky. Purely visual for now.

## Files

- `crates/shared/src/weather.rs`: `WeatherField`, `WeatherSample`, private `Simplex4`. Golden, range, wrap and variety tests.

## Public API

- `WeatherField::new(&WorldParams)`, `.sample(x, y, hours) -> WeatherSample` (`hours`: fractional game hours since the world began).
- `WeatherSample { cover, rain }`, both `0..=1`.

## Depends on

- `shared::rng`, `shared::world`, `libm`.

## Used by

- Client `common` (`CurrentWorld::weather`), `world_map::sky`, `dungeon_map::rain`.

## Notes

- `Simplex4` is hand-written, since `fastnoise-lite` stops at 3D and the east-west wrap already uses three dimensions (time is the fourth). Only `+ - * /` and `floor`, seeded by shuffling its table.
- Single clouds on top of larger, slower fronts, with the whole sky drifting east (`WIND`).
- Rain or snow isn't decided here: the client combines `rain` with `shared::climate`'s temperature.
- Takes the world's clock, a float that only counts up (`ServerClock` on the client).
