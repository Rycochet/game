# shared: climate

Temperature, moisture and the biome they make, at any world map position, following the Whittaker diagram. Comes from the public terrain seed, so the client works it out itself. In `shared` because biomes will affect rules later (mobs, feature kinds, costs); for now only the client's colours use it.

## Files

- `crates/shared/src/climate.rs`: `Climate`, `ClimateSample`, `Biome`. Golden, wrap and variety tests.

## Public API

- `SNOW_TEMPERATURE`: land colder than this is snow, around height 90 on average.
- `Climate::new(&WorldParams)`, `.sample(x, y, height) -> ClimateSample`.
- `ClimateSample { temperature, moisture }` (both `0..=1`), `.biome()`.
- `Biome`: `Tundra`, `BorealForest`, `Grassland`, `Woodland`, `SeasonalForest`, `TemperateRainforest`, `Desert`, `Savanna`, `TropicalRainforest`; `ALL`; `Biome::of(temperature, moisture)`.

## Depends on

- `shared::world`, `shared::rng` (through `WorldParams::seed`), `fastnoise-lite`, `libm`.

## Used by

- Client `common` (`CurrentWorld::climate`), `surface_style` (colours), `world_map` (`HeightCache::climate`; `sky`: temperature decides rain or snow), `dungeon_map` (ground colour outside, rain or snow).

## Notes

- Temperature: `y = 0` is the equator and the poles are a quarter of the planet's width away; planets are huge next to the 400-point view, so slow noise (`ZOOM` 700, 3 layers) does most of the local work. The world's `warmth` shifts it everywhere. Height cools it by the height itself rather than above the sea (`HEIGHT_COOLING`, unlimited), so hills pass through colder biomes and on to snow at about the same height on every world.
- Constants were tuned so snow covers about as much land as a fixed height of 90 did, over several worlds (a test keeps it within a factor of two).
- Noise is stretched (`STRETCH`) because simplex values bunch around 0, which would make nearly everywhere mild.
- Changing any constant here changes colours everywhere (golden test), but not walking or features, so it needs no `WORLD_MAP_VERSION` bump until rules use biomes.
