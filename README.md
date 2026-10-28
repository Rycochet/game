# Hall of Worlds

A multiplayer roguelike with an infinite, procedurally generated world.

- <https://hallofworlds.com>: landing page and email (not live yet)
- <https://hallofworlds.app>: the game, running the Docker image (not live yet)
- <https://hallofworlds.wiki>: game documentation (not live yet; planned to live in this repo)

## Development

The game is written in [Rust](https://www.rust-lang.org/) using the [Bevy](https://bevy.org/) game engine. It has two parts:

- **Client** (`hall-of-worlds`): the game itself, as a native app (Windows, macOS, Linux) or WebAssembly in the browser. Native clients include the server for single player; the browser client only connects to a server.
- **Server** (`hall-of-worlds-server`): the authoritative game server, native only. The Docker image runs it alongside the web client.

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
| Run the standalone server | `cargo run -p server`   |                                            |
| Run in a browser          | `trunk serve`           | Task "Serve (web)", then <http://localhost:3000/> |
| Release build (native)    | `cargo build --release` | Task "Release build (native)"             |
| Release build (web)       | `trunk build --release` | Task "Release build (web)", output in `dist/` |
| Test (what CI runs)       | `cargo xtask test`      | Task "Test"                               |
| Lint                      | `cargo clippy`          | Shown automatically in the editor         |
| Format                    | `cargo fmt`             | Automatic on save                         |

The first build downloads and compiles Bevy, which takes several minutes. Later builds are much faster.

`cargo xtask test` checks formatting, type-checks/lints the native and web builds with warnings treated as errors, and runs the tests. GitHub runs it on every branch (the "Test" workflow) and it must pass before merging.

Source code is in `crates/`: `shared` holds game rules and world generation used by both sides, `client` is the game app, `server` is the game server. Images/sounds/fonts go in `crates/client/assets/`. In the game, click the map or use the arrow keys to move; G and T start demo modes. Build output goes in `target/` (native) and `dist/` (web), both local only.

### Docker

The `Dockerfile` builds an x86-64 image with the game server and the web client (served on port 8080):

```bash
docker build -t hall-of-worlds .
docker run -p 8080:8080 hall-of-worlds
```

Every merge to `master` publishes it as `ghcr.io/rycochet/hall-of-worlds:latest`.

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

## Licence

Hall of Worlds is licensed under [Creative Commons Attribution-NonCommercial 4.0 International](https://creativecommons.org/licenses/by-nc/4.0/) (CC BY-NC 4.0): you may share and adapt it with attribution, but not for commercial purposes. See [LICENSE](LICENSE). Third-party dependencies keep their own licences.
