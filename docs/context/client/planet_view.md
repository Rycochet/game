# client: planet_view

View Planet: the whole world as a turning globe, a day a minute, with its moons going round it, over everything until a click, tap or Esc. A debug view for now (the debug menu's View Planet), planned as the loading screen for travelling between planets. Only for looks: nothing here is a rule, and the server knows nothing of it.

## Files

- `crates/client/src/planet_view/mod.rs`: `PlanetViewPlugin`, `ViewPlanet`, `PLANET_LAYER` (8), `HOURS_PER_SECOND` (a day a minute).
- `open.rs`: `PlanetView` (resource: `showing`, with the layer, scene, texture, rows filled, the water and snow lines, the hour and real time on opening), `open` (on `ViewPlanet`, if there is a `CurrentWorld`): the black UI layer (`widgets::layer_z(50)`, above the menus) with the picture as large as fits and a note, the texture and the scene.
- `globe.rs`: `Scene`, `spawn_scene`: the globe (Bevy's UV sphere tipped so north is +Y), a `DirectionalLight` per sun (colour, `SUN_LUX` × strength), the moons (size × `MOON_RADIUS`, textured, tinted by colour), and a camera drawing into the picture (order -50, black, `Tonemapping::None`, a little ambient), far enough back for the furthest moon.
- `sky.rs`: the maths, tested alone: `equator`, `from_ground` (`[east, up, south]` at a point of the equator to a direction round the globe), `Sky` (`ground_angle`, `globe_turn`, `sun_direction`, `moon_place`), `moon_distance`.
- `texture.rs`: `texture_size`, `ground_at`, `new_texture`, `fill_texture` (rows within `FRAME_BUDGET`, 6 ms a frame).
- `spin.rs`: `spin`: the view's own clock, then the globe, lights and moons.
- `moon_look.rs`: `MoonLook` (`of(terrain seed, moon number)`: style, own axis and turn, `turn`, `image`: 128×64 greys); in `moon_look/`: `pattern.rs` (cratered, banded, cracked ice, patchy, from value noise on the sphere).
- `input.rs`: `take_input` (`PreUpdate`, after `PointerSystems`, before `WidgetSystems::Presses`); `close.rs`: `close`, `close_without_world`.

## Public API

- `PlanetViewPlugin`; `ViewPlanet` (message): show the planet.

## Depends on

- `common` (`CurrentWorld`, `ServerClock`, `Weather`, `PointerSystems`, `TouchGesture`), `surface_style` (`SurfaceStyle`, `SurfaceKind`), `widgets` (`layer_z`, `WidgetSystems`), `shared::celestial` (`at`, `bodies`, `period`, `sun_strength`), `shared::surface`, `shared::rng`.

## Used by

- `debug_menu` (View Planet).

## Notes

- The picture is equirectangular: columns are `x` round the world, rows `y` from `-width / 4` to `width / 4`, the climate's poles (the world goes on past them; the globe can't show it). At most 1024 × 512; a smaller world gets a pixel a grid point. Each pixel is `SurfaceStyle::ground_colour` at mid-tide (`sea_level`) and the snow line on opening, darker under trees. From `BROAD_FROM` (4) grid points a pixel only `Surface::broad_height` is used, as finer layers only speckled it. It fills in as the globe turns (about 5 s on a debug build).
- What it shows of the generator (owner's reason for it): the noise is sampled on a cylinder, so near the poles features squeeze into a pinch; and its largest layer is about 256 grid units, small next to an Earth-sized world, so there is nothing continent-sized.
- The sky: the main sun stays put about 60° right of the camera, and the globe turns under it, towards the east if suns rise in the east, with `x = 0` facing it at noon (no time zones, so that is where the clock is true). Other suns are where they are seen from `x = 0`. Moons' orbits ignore the planet's turning, so they stay put in space (`from_ground` at the noon angle); distances follow Kepler (period to the 2/3) squeezed into 1.7 to 4 radii. Moon phases come from the lights, not `BodyNow::lit`.
- While open it takes all input, so Esc never reaches the system menu and clicks never reach the map; presses in the first 0.25 s are ignored, so the debug menu's own press can't close it. It closes by itself if the world goes (the connection lost).
- Planned (PR 2): coarse clouds for looks only, a see-through shell drawn by its own shader, changing over time by moving through the noise and by bands of latitude drifting at different speeds.
