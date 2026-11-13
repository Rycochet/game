# shared: hall

The Hall of Worlds: a special place with fixed rules, made by no generator. A marble walkway hovering in a void, `HALL_LENGTH` (2^33) tiles east to west and coming round again, with a World Gate every 4 tiles on both sides leading to one of the 16 World Gates of one planet. What each tile is, where each gate leads (both ways), what it is built like and its code of runes. Players can be in it (dev tools take them there); the client draws it (`dungeon_map`, `hall_view`).

## Files

- `crates/shared/src/hall/mod.rs`: the constants (`HALL_LENGTH`, `WALKWAY_ROWS` 3, `GATE_SPACING` 4, `PLANET_BITS` 28, `PLANETS`), `pub use`s. In `hall/`: `tile.rs` (`Tile`, `tile`, `wrap_x`, `hue`), `place.rs` (`GatePlace`, `Side`), `address.rs` (`GateAddress`), `shuffle.rs` (the per-sixteenth multipliers, steps and offsets), `design.rs` (`GateDesign`), `code.rs` (`GateCode`, `RUNES`), `level.rs` (`cell`, `level`, `LEVEL_TOP`, `LEVEL_ROWS`), `glyph.rs` (`RUNE_STROKES`, `SEPARATOR_STROKES`, `Stroke`, `GLYPH_WIDTH`, `GLYPH_HEIGHT`).

## Public API

- `tile(x, y) -> Tile::{Walkway, Gate(Side), Void}`: rows `0..3` walkway (`WALKWAY_ROWS` 3, owner); rows -1 (north) and 3 (south) a gate's floor every 4th column, void between; void beyond. `wrap_x` brings any `x` into `0..HALL_LENGTH`. `hue(x)`: degrees, once round over the whole Hall (looks only).
- `level(min_x, width)`: a stretch of the Hall as a `dungeon::Level`, rows `LEVEL_TOP` (-2) to 4 (`LEVEL_ROWS` 7: a row of void beyond each gate's), `x` unwrapped; `cell(x, y)`: walkway `Floor`, a gate's floor `WorldGate`, void `Hole`. Steps are checked on it by both sides (`movement::step_on_level`).
- `GatePlace { column, side }`: `from_number(u32)` / `number()` (`2 × column + side`, north first: every `u32` is one gate), `at(x, y)`, `x()`, `y()`, `front()` (the walkway tile in front), `address()`, `design()`, `code()`.
- `GateAddress`: `new(seed, entrance)` (`None` past `PLANETS` or 16), `for_world(WorldSeed, entrance)`, `from_bits` / `bits()` (`entrance << 28 | seed`), `seed()`, `entrance()` (the order `WorldGates::find` gives), `world()`, `place()`.
- `GateDesign` (`RoundArch`, `SquareLintel`, `PointedArch`, `HorseshoeArch`, `Ogee`, `Trefoil`, `SteppedCorbel`, `Keyhole`; `ALL`, `of(place)`, `index()`): from a hash of the place.
- `GateCode([u8; 7])` (`Serialize`): `of(place)`, `of_planet(WorldSeed)` (a planet's own name in runes: its first World Gate's code; `None` off the Hall's planets), `place()` (`None` if the design doesn't match: a typo), `parse(text)` (ignores spaces and `SEPARATOR`), `Display` (3 runes, `SEPARATOR` ᛫, 4 runes). `RUNES` (32), `CODE_LENGTH` (7). `RUNE_STROKES`: each rune as straight strokes on a 4 × 6 grid (`y` down), `SEPARATOR_STROKES` the dot: what every rune shown anywhere is drawn from, never a font (owner), so they are identical (tests: on the grid, no two alike).

## Depends on

- `shared::world` (`WorldSeed`), `shared::world_gate` (`WORLD_GATES`), `shared::rng`, `shared::dungeon` (`Level`, `Cell`).

## Used by

- `server` (`hall`: steps, arriving, dev tools), client `site` (`hall`: the stretch round the player), `hall_view` (gates, runes), `dungeon_map` (`hue`).

## Notes

- A planet's seed is its own number (any seed under 2^28), so the world's `DEFAULT_WORLD_SEED` (12345) is a Hall planet. Seeds are public here (owner).
- Entrance `k` of every planet is in the `k`-th sixteenth of the Hall (gate numbers `k << 28 ..`), at `(MULTIPLIERS[k] × seed + OFFSETS[k]) mod 2^28` within it; `STEPS[k]` (odd, the multiplier's inverse) undoes it and is how much the seed changes from one gate number to the next. Gates up to 3 numbers apart stand beside each other (same or next column), so the tests check that `±1..3 × STEPS[k]` never repeats between sixteenths (two planets are neighbours at most once in the whole Hall) and that neighbours differ at every sixteenth's edge and across the seam. The tables are `const fn`s of fixed salts.
- Codes: the 35-bit value `design << 32 | address bits`, 5 bits a rune, most significant first. Futhorc order, without æsc (ᚫ), the easiest to mistake for ac (ᚪ). Golden test: seed 12345's first gate is number 175,562,835, a pointed arch, ᚻᚠᚠ᛫ᚠᛇᚢᚣ (codes are the 35-bit `design << 32 | address`, so low seeds start with ᚠ, 0). Changing a constant moves every gate: only on purpose.
