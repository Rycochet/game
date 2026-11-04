# shared: climate

Temperature, moisture and the biome they make, at any world map position, following the Whittaker diagram. Comes from the public terrain seed, so the client works it out itself. In `shared` because biomes will affect rules later (mobs, poi kinds, costs); for now only the client's colours use it.

## Files

- `crates/shared/src/climate.rs`: `Climate`, `ClimateSample`, `Biome`. Golden, wrap, variety, lapse rate, planet and belt tests.
- `climate/planet.rs`: the climate of surfaces in metres from the planet: `Sunlit` (tilt and the suns' heat), `polar_limit`, `moisture_belt`, `ICE_LATITUDE`, `SEA_ICE`.

## Public API

- `SNOW_TEMPERATURE`: land colder than this is snow, around height 90 on average.
- `Climate::new(&WorldParams)`, `.sample(x, y, height) -> ClimateSample`, `.sample_metres(x, y, metres)` (for surfaces worked out in metres: the planet's own climate, below; the air cools at Earth's 6.5 °C a km from the sea).
- `ICE_LATITUDE` (80): always ice past it (its edge wanders 3° with the local temperature); `SEA_ICE` (-6 °C): the sea is frozen below this yearly average; `moisture_belt(latitude)` (`-1..=1`, for poi kinds later).
- `CELSIUS_AT_ZERO` (-10.5), `CELSIUS_PER_UNIT` (50): the scale in °C, so snow is about -8 °C and the equator's sea about 27 °C; `ClimateSample::celsius()`.
- `ClimateSample { temperature, moisture }` (both `0..=1`), `.biome()`.
- `Biome`: `Tundra`, `BorealForest`, `Grassland`, `Woodland`, `SeasonalForest`, `TemperateRainforest`, `Desert`, `Savanna`, `TropicalRainforest`; `ALL`; `Biome::of(temperature, moisture)`.

## Depends on

- `shared::world`, `shared::rng` (through `WorldParams::seed`), `fastnoise-lite`, `libm`.

## Used by

- Client `common` (`CurrentWorld::climate`), `surface_style` (colours), `world_map` (`HeightCache::climate`; `sky`: temperature decides rain or snow), `dungeon_map` (ground colour outside, rain or snow).

## Notes

- Surfaces in metres (2026-11-04, owner's design): the world's `tilt` is an angle of 15° to 35° (tuned so ice comes naturally near 80°); each latitude's yearly sunlight for it (the second Legendre term of the yearly average: Earth's tilt gives about 29 °C at the equator and -14 °C at the poles, after flattening for the heat winds and seas carry); the suns' heat (each sun's size squared, companions half) scales it from absolute zero (`HEAT_POWER` 0.06: four times Earth's sun is about 25 °C warmer); the world's `warmth` only a little (±5 °C), noise ±6 °C. A polar limit falls to -8 °C at `ICE_LATITUDE` and -20 °C at the pole, with always ice past it. Moisture: the world's wetness, the belts of rain (wet equator, dry 30°, wet 60°, dry poles) and less noise than `Classic`'s. `Classic` (`sample`) is unchanged.
- Temperature (`Classic`): `y = 0` is the equator and the poles are a quarter of the planet's width away; planets are huge next to the 400-point view, so slow noise (`ZOOM` 700, 3 layers) does most of the local work. The world's `warmth` shifts it everywhere. Height cools it by the height itself rather than above the sea (`HEIGHT_COOLING`, unlimited), so hills pass through colder biomes and on to snow at about the same height on every world.
- Constants were tuned so snow covers about as much land as a fixed height of 90 did, over several worlds (a test keeps it within a factor of two).
- Noise is stretched (`STRETCH`) because simplex values bunch around 0, which would make nearly everywhere mild.
- Changing any constant here changes colours everywhere (golden test), but not walking or pois, so it needs no `WORLD_MAP_VERSION` bump until rules use biomes.
