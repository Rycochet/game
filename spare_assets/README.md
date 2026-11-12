# Spare assets

Art kept in the repository but not shipped with the game: only `crates/client/assets/` is. Anything the game starts to use is copied (or drawn) from here into the client's assets, and credited in `crates/client/assets/ATTRIBUTION.md`.

- `monsters/`, `terrain/`: David E. Gervais's Dungeon Odyssey tiles, the whole sets (54 × 54); `angbandtk/`: his AngbandTk tiles split from their sheets (32 × 32), which `tools/make_town_atlas.py` reads. The client's assets hold copies of only those the code names: to use another, copy it into the same folder there.
- `game-icons/`: every icon from [game-icons.net](https://game-icons.net), the SVGs from <https://github.com/game-icons/icons> (white on black), unchanged, one folder per author, with `license.txt` (CC BY 3.0, or CC0 where marked). List the ones the game uses in `xtask/icons.txt` and run `cargo xtask icons` to draw them into `crates/client/assets/icons/`.
- `makehuman/`: [MakeHuman](http://www.makehumancommunity.org)'s base mesh of a person (CC0) and a few of JALdMIC's animal shapes for it (CC-BY), unchanged, with their sources in its `README.md`. `cargo xtask shapes` measures creatures' proportions from them; the game uses only the figures.
