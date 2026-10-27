# game

## Development

The game is written in [Rust](https://www.rust-lang.org/) using the [Bevy](https://bevy.org/) game engine. It builds as a native app (Windows, macOS, Linux) and as WebAssembly for the browser.

### One-time setup (Windows)

1. Install the Visual Studio C++ Build Tools (with the "Desktop development with C++" workload): <https://visualstudio.microsoft.com/visual-cpp-build-tools/>
2. Install Rust: `winget install Rustlang.Rustup` (or <https://rustup.rs/>), then open a new terminal.
3. Install Trunk, the web build tool: `cargo install --locked trunk`
4. Install Visual Studio Code <https://code.visualstudio.com/> and accept the recommended extensions when you open this folder.

`rust-toolchain.toml` makes Rust download the correct compiler and WebAssembly target automatically on the first build.

On Linux you also need some system libraries, see <https://bevy.org/learn/quick-start/getting-started/setup/>.

### Running

| What                      | Command                 | VSCode                                     |
| ------------------------- | ----------------------- | ------------------------------------------ |
| Run natively              | `cargo run`             | Task "Run (native)", or F5 to debug        |
| Run in a browser          | `trunk serve`           | Task "Serve (web)", then <http://localhost:3000/> |
| Release build (native)    | `cargo build --release` | Task "Release build (native)"             |
| Release build (web)       | `trunk build --release` | Task "Release build (web)", output in `dist/` |
| Test (what CI runs)       | `cargo xtask test`      | Task "Test"                               |
| Lint                      | `cargo clippy`          | Shown automatically in the editor         |
| Format                    | `cargo fmt`             | Automatic on save                         |

The first build downloads and compiles Bevy, which takes several minutes. Later builds are much faster.

`cargo xtask test` checks formatting and type-checks/lints the native and web builds with warnings treated as errors. GitHub runs it on every branch (the "Test" workflow) and it must pass before merging.

Source code is inside `src/`, images/sounds/fonts go in `assets/`. Build output goes in `target/` (native) and `dist/` (web), both local only.

## Overview

Multiplayer roguelike with infinite world

## Details

### World

1. The world has a size, which decides how many horizontal and vertical grids exist, each grid unit is 1km (Earth is 40km around).
    * The poles are not approachable beyond a certain Y (because it's hard to get around vertically).
    * Poles get their value moved towards deep water (-1) the closer you get, temperature decides whether that is ice or water
1. The world has an "orbit", this decides how hot/cold the world is, hot gets more sand.
1. The world has a "density", this is used to specify how common metal ores are and what depth they can be found.
1. The world has a "tilt", this is used to specify how extreme the weather change is (colder further north etc).
    * Orbit and density are used together to decide how long the year is (all orbits are elliptical).
    * Year and tilt are used to decide how long the seasons are etc.
1. The world has a "climate", this decides how wet/dry the world is in general, plant growth speed etc.

### Perlin noise generation

1. Terrain uses a 3d `terrain-seed` for ground level and gross terrain (deep water < 0.25 < water < 0 < sand < 0.1 < grass < 0.75 < mountain).
    * Ground level is Z=0, this is used for the type being displayed as above.
    * Below ground level is negative Z, each "layer" is -0.1 on the Z.
        * Type is `Math.MAX(value, Math.min(0, -Z))`, however anything below Z=-0.25 is "deep water" by definition.
        * "grass" becomes "dirt", "mountain" becomes "rock" for underground.
        * Metal ores can be found at deeper levels, only in rock, and the chance of it appearing is based on the value (closest to 1, weight by world seed).
1. Wetness uses a 2d `wet-seed` for detailed terrain using a http://pcg.wikidot.com/pcg-algorithm:whittaker-diagram
1. Landscape (decoration) uses a 3d `<terrain>-landscape-seed`, the Z is used for date based changes over time (sine wave Z).
    * Each terrain has a different seed, so plants don't follow lines.
1. Weather uses a 3d `weather-seed`.
    * The Z axis is used for time, with 0 being "start of game" and increasing linearly (1 hour per unit?).
    * Weather is adjusted by in-game date (colder weather for snow etc)
    * Weather adjusts shading on the cells.
1. Buildings, whether on the surface or underground are per grid (X,Y,0) using a `<grid>-settlement-seed`
    * Each grid uses the first number for how many (0-3?)
    * Each settlement uses a subsequent pair of numbers for X & Y ccordinates within the grid, together with the type.
    * Terrain and landscape is used to decide on the gross type of settlement (house / village / town / city / castle / dungeon).

### Persistence

* Every grid can be changed by the players.
* Some changes are temporary (growth / monsters /doors etc), and will be forgotten / reset after a period of time (1 hour?).
* Some changes are per-player (unique monster rewards / quests etc) and must be saved for the individual player.
* Some changes are permanent (digging etc) and must be saved for everyone.
