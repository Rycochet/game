# shared: celestial

A world's suns and moons, and where they are at any time, from the public terrain seed. They make light, and the moons pull the tides: nothing draws them. Clients work this out themselves, as they do the weather, so there are no messages and no generator version. The server does too, for the tide and for quests that wait for a moon in the sky (`quest::Need::Moon`).

## Files

- `crates/shared/src/celestial.rs`: `Celestial`, `Body`, `BodyNow`, `BodyKind`, `ColourName`, `flat_light`, `brightest`, `MOST_BODIES`. Golden and chance tests. In `celestial/`: `celestial.rs` (`from_seed`), `body.rs`, `colour_name.rs`, `now.rs` (`at`), `period.rs`, `light.rs`, `sun.rs`, `moon.rs`, `moon_path.rs`, `vector.rs`, `dev_tools.rs`, `tide.rs` (`tide_pull`).

## Public API

- `Celestial::from_seed(terrain seed)`.
- `bodies() -> Vec<Body>`: every sun, then every moon, whatever the time: `kind`, `colour` (linear RGB), `colour_name` (`Option<ColourName>`: `None` when too near white to have a name), `size`. Use this to ask what a world has.
- `at(hours) -> Vec<BodyNow>`: the same order, at a time: `body`, `direction` (unit, `[east, up, south]`), `lit` (how much of a moon's face we see is lit), `waxing`, `along` (how far along its way across the sky: negative before its top, positive after, always rising, whichever way that goes on the world), `phase()` (`MoonPhase`: for later, as nothing else reads the name yet), `light` (linear RGB, faded near the horizon, zero below it), `is_up()`, `flat_light()`. Everything goes through this, so what is said matches what is seen.
- `sun_count()`, `moon_count()`: how many of each (in `bodies` and `at` the moons come after the suns); the quests' moon needs go by them. `ColourName` is serde, for quest hints.
- `period(number) -> Option<f64>`: hours one of `bodies` takes to go round (a sun's day, a moon's orbit); `client::planet_view` places moons by it.
- `set_colour(number, colour)`, `sun_strength(number)`, `set_sun_strength(number, strength)`, `colour_along(kind, 0..=1)`: for the debug menu's panel.
- `tide_pull(hours) -> f64`: how far the moons pull the sea, `-1..=1` (0 with no moons), for `WorldParams::tide`.
- `flat_light(&bodies)`: their light together on flat ground. `brightest(&bodies)`: the index of the one lighting flat ground most.

## Depends on

- `shared::rng`, `libm`.

## Used by

- `shared::world` (the tide); `server` (`ServerWorld`, for the water level); client `common` (`CurrentWorld::celestial`, `SkyNow`), `world_map` (`lights`, baked slope shading, water level), `planet_view` (the globe's sky).

## Notes

- Suns (owner's design): 5% of worlds have two, 0.1% three. The main sun has 12 to 16 hours of daylight, noon at hour 12, rising in the east or west by world. Colours run blue, white, yellow, orange, with a brightness pulling them towards white (through white, or blue to yellow would pass through green); suns in one world must differ. Bluer means smaller and quicker: a companion's period depends on its colour compared with the main sun, so it drifts through the days. The main sun is always exactly 24 hours, whatever its colour, so the clock stays true. Companions also have their own noon and are weaker.
- A sun's path: `[sin, cos + rise, lean]`, normalised. `rise` sets the hours above the horizon; `lean` (15 to 35 degrees, north or south) keeps noon from being straight overhead, so the terrain never loses its shading. The main sun's strength is scaled so noon gives exactly its colour on flat ground.
- Moons (owner's design): 0 to 5, 1 the most common and 0 the least (`MOON_CHANCES`). Each has an orbit of 7 to 90 hours about any axis, except near upright, so it always climbs at least 20 degrees. Their orbits ignore the planet's turning. Light uses real phases: each sun lights it by `(1 − moon·sun) / 2`, also from below the horizon, times its colour and `size²`, so a moon opposite a sun is full. No eclipses.
- Tides (owner's design): the moons' heights in the sky (the up part of their direction: 1 at the zenith, -1 at the nadir), averaged by mass (`size³`), below the horizon too. One moon follows its true orbit, so one that never climbs far makes gentle tides; moons high together make the highest tides, and one up with another down cancel. The average never passes ±1, so `high_water` holds. Orbit speeds set how often tides come; the water still moves once a game hour.
- All light fades in over the lowest 15 degrees or so (`FADE_HEIGHT`), about an hour of the sun's climb.
- No time zones (decided for simplicity): the whole planet has one time of day, like the tides.
- Kept apart from `WorldParams`, whose debug text is a golden value.
- Colour names come from a body's own colour, not what it reflects now: the blue moon stays blue under an orange sun.
