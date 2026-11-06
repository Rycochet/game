# Spare assets

Art kept in the repository but not shipped with the game: only `crates/client/assets/` is. Anything the game starts to use is copied (or drawn) from here into the client's assets, and credited in `crates/client/assets/ATTRIBUTION.md`.

- `game-icons/`: every icon from [game-icons.net](https://game-icons.net), the SVGs from <https://github.com/game-icons/icons> (white on black), unchanged, one folder per author, with `license.txt` (CC BY 3.0, or CC0 where marked). List the ones the game uses in `xtask/icons.txt` and run `cargo xtask icons` to draw them into `crates/client/assets/icons/`.
