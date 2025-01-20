# game

## Development

1. Install NodeJS <https://nodejs.org/en>
2. Install Visual Studio Code <https://code.visualstudio.com/> as an editor
3. Open a terminal within VSCode and run `npm ci`
4. ...
5. Profit

To run this open a terminal and run `npm start`, this will allow you to open the app in a browser window on <http://localhost:3000/>, this will automatically update on code changes.

Source code is inside `src/`, output code in `dist/` (local only), anything inside `public/` will automatically be copied over.

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
