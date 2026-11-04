# shared: hall

The Hall of Worlds: a special place with fixed rules, made by no generator. A marble walkway hovering in a void, `HALL_LENGTH` (2^33) tiles east to west and coming round again, with a World Gate every 4 tiles on both sides leading to one of the 16 World Gates of one planet. What each tile is, where each gate leads (both ways), what it is built like and its code of runes. Not yet a place players can be in (PR 2) or drawn (PR 3).

## Files

- `crates/shared/src/hall/mod.rs`: the constants (`HALL_LENGTH`, `WALKWAY_ROWS` 4, `GATE_SPACING` 4, `PLANET_BITS` 28, `PLANETS`), `pub use`s. In `hall/`: `tile.rs` (`Tile`, `tile`, `wrap_x`, `hue`), `place.rs` (`GatePlace`, `Side`), `address.rs` (`GateAddress`), `shuffle.rs` (the per-sixteenth multipliers, steps and offsets), `design.rs` (`GateDesign`), `code.rs` (`GateCode`, `RUNES`).

## Public API

- `tile(x, y) -> Tile::{Walkway, Gate(Side), Void}`: rows `0..4` walkway; rows -1 (north) and 4 (south) a gate's floor every 4th column, void between; void beyond. `wrap_x` brings any `x` into `0..HALL_LENGTH`. `hue(x)`: degrees, once round over the whole Hall (looks only).
- `GatePlace { column, side }`: `from_number(u32)` / `number()` (`2 × column + side`, north first: every `u32` is one gate), `at(x, y)`, `x()`, `y()`, `front()` (the walkway tile in front), `address()`, `design()`, `code()`.
- `GateAddress`: `new(seed, entrance)` (`None` past `PLANETS` or 16), `for_world(WorldSeed, entrance)`, `from_bits` / `bits()` (`entrance << 28 | seed`), `seed()`, `entrance()` (the order `WorldGates::find` gives), `world()`, `place()`.
- `GateDesign` (`RoundArch`, `SquareLintel`, `PointedArch`, `HorseshoeArch`, `Ogee`, `Trefoil`, `SteppedCorbel`, `Keyhole`; `ALL`, `of(place)`, `index()`): from a hash of the place.
- `GateCode([u8; 7])`: `of(place)`, `place()` (`None` if the design doesn't match: a typo), `parse(text)` (ignores spaces and `SEPARATOR`), `Display` (3 runes, `SEPARATOR` ᛫, 4 runes). `RUNES` (32), `CODE_LENGTH` (7).

## Depends on

- `shared::world` (`WorldSeed`), `shared::world_gate` (`WORLD_GATES`), `shared::rng`.

## Used by

- Nothing yet.

## Notes

- A planet's seed is its own number (any seed under 2^28), so the world's `DEFAULT_WORLD_SEED` (12345) is a Hall planet. Seeds are public here (owner).
- Entrance `k` of every planet is in the `k`-th sixteenth of the Hall (gate numbers `k << 28 ..`), at `(MULTIPLIERS[k] × seed + OFFSETS[k]) mod 2^28` within it; `STEPS[k]` (odd, the multiplier's inverse) undoes it and is how much the seed changes from one gate number to the next. Gates up to 3 numbers apart stand beside each other (same or next column), so the tests check that `±1..3 × STEPS[k]` never repeats between sixteenths (two planets are neighbours at most once in the whole Hall) and that neighbours differ at every sixteenth's edge and across the seam. The tables are `const fn`s of fixed salts.
- Codes: the 35-bit value `design << 32 | number`, 5 bits a rune, most significant first. Futhorc order, without æsc (ᚫ), the easiest to mistake for ac (ᚪ). Golden test: seed 12345's first gate is number 175,562,835, a pointed arch, ᚻᚳᚹ᛫ᛈᚪᚦᛗ. Changing a constant moves every gate: only on purpose.
