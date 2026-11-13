# Attribution

Artwork in this folder made by other people, with the licence it is used under. Placeholder 3D images are by Claude Code (Opus 5.5).

## `fonts/HallOfWorldsTitle.ttf`: Cinzel Decorative Bold by Natanael Gama, cut down

- Designer: **Natanael Gama**
- Source: Google Fonts, <https://github.com/google/fonts/tree/main/ofl/cinzeldecorative>
- Licence: [SIL Open Font License 1.1](https://openfontlicense.org), in `fonts/HallOfWorldsTitle-OFL.txt`
- Changes: cut down to the letters of the game's title, as it is only used for that, and renamed "Hall of Worlds Title" (the licence reserves the name "Cinzel" for unchanged copies). Made from `CinzelDecorative-Bold.ttf` with fontTools: `pyftsubset CinzelDecorative-Bold.ttf --text="Hall of Worlds" --no-hinting --name-IDs='*' --drop-tables+=DSIG,FFTM`, then name IDs 1, 3, 4 and 6 changed to the new name and a note added to 5. Any other letter in the title needs it made again.

## `fonts/Lexend-Variable.ttf`: Lexend by The Lexend Project Authors

- Designers: **Bonnie Shaver-Troup, Thomas Jockin, Santiago Orozco, Héctor Gómez, Superunion**
- Source: Google Fonts, <https://github.com/google/fonts/tree/main/ofl/lexend>
- Licence: [SIL Open Font License 1.1](https://openfontlicense.org), in `fonts/Lexend-OFL.txt`
- Changes: none, but the file renamed from `Lexend[wght].ttf`.

## `fonts/NotoEmoji.ttf`: Noto Emoji by Google

- Copyright 2013 Google Inc.
- Source: Noto Emoji 1.05, <https://github.com/googlefonts/noto-emoji>, as included with the `bevy_replicon_renet2` crate
- Licence: [SIL Open Font License 1.1](https://openfontlicense.org), in `fonts/NotoEmoji-OFL.txt`
- Changes: none, but the file renamed from `NotoEmoji-Regular.ttf`.

## `icons/`: game-icons.net icons by their contributors

- Artists: the contributors to game-icons.net, each in their own folder: **Lorc**, **Delapouite**, **John Colburn**, **Felbrigg**, **John Redman**, **Carl Olsen**, **Sbed**, **PriorBlue**, **Willdabeast**, **Viscious Speed** (CC0), **Lord Berandas**, **Irongamer**, **HeavenlyDog**, **Lucas**, **Faithtoken**, **Skoll**, **Andy Meneely**, **Cathelineau**, **Kier Heyl**, **Aussiesim**, **Sparker**, **Zeromancer** (CC0), **Rihlsul**, **Quoting**, **Guard13007**, **DarkZaitzev**, **SpencerDub**, **GeneralAce135**, **Zajkonur**, **Catsu**, **Starseeker**, **Pepijn Poolman**, **Pierre Leducq**, **Caro Asercion**, **SeregaCthtuf**. Icons made by them, from <https://game-icons.net>.
- Source: the SVGs in <https://github.com/game-icons/icons>, kept unchanged with their `license.txt` in `spare_assets/game-icons/` (never shipped).
- Licence: [Creative Commons Attribution 3.0](http://creativecommons.org/licenses/by/3.0/) (CC BY 3.0), or CC0 where marked.
- Changes: `cargo xtask icons` draws the ones listed in `xtask/icons.txt` as 128 × 128 PNGs, white on a transparent background (the black square behind each left out), at `icons/<author>/<name>.png`.

## Animals' body shapes (figures in the code): measured from JALdMIC's MakeHuman animals

Not a file here, but figures in `crates/shared/src/body/measured.rs` and `measured_heads.rs`: the outlines of the cow's, dog's and horse's trunks (and so the wolf's and the centaur's lower body), and of the wolf's, cow's, dog's and horse's heads, measured from these models.

- Artist: **JALdMIC**, animal targets for MakeHuman's base mesh of a person (by the MakeHuman team, CC0)
- Source: MakeHuman's `animal04` (bodies) and `animal03` (heads) asset packs, <https://static.makehumancommunity.org/assets/index.html>; the files used are kept unchanged in `spare_assets/makehuman/` (never shipped), with their links in its `README.md`
- Licence: [Creative Commons Attribution](https://creativecommons.org/licenses/by/4.0/) (CC BY)
- Changes: only measurements are used: `cargo xtask shapes` applies each target to the base mesh and measures rings along the trunk (in thousandths of the height at the withers) and along the head (in thousandths of its length), which are scaled to the game's own animals. No mesh is shipped or read by the game.

## `monsters/`: Dungeon Odyssey monster tiles by David E. Gervais

- Artist: **David E. Gervais**
- Source: the D. Gervais tiles library, <http://pousse.rapiere.free.fr/tome/> ("DO Monsters" download)
- Licence: [Creative Commons Attribution 3.0](http://creativecommons.org/licenses/by/3.0/) (CC BY 3.0)
- Changes: the solid magenta (`#ff00ff`) background was made transparent, the images converted to RGBA PNG, and the file and folder names lower-cased. Only the tiles the game uses are here; the whole set is in `spare_assets/monsters/` (never shipped).

## `terrain/` (none in use): Dungeon Odyssey terrain tiles by David E. Gervais

- Artist: **David E. Gervais**
- Source: the D. Gervais tiles library, <http://pousse.rapiere.free.fr/tome/> ("DO Terrain" download)
- Licence: [Creative Commons Attribution 3.0](http://creativecommons.org/licenses/by/3.0/) (CC BY 3.0)
- Changes: the solid magenta (`#ff00ff`) background was made transparent, the images converted to RGBA PNG, and the file and folder names lower-cased. None are used now, so the whole set is only in `spare_assets/terrain/` (never shipped).

## `town_atlas.png`: from the AngbandTk tiles below

Made by `tools/make_town_atlas.py` from those tiles (same artist and licence): some tinted, and four shop signs (saw, scissors, hide, gem) drawn over a door by Claude.

## `angbandtk/`: AngbandTk flat tiles by David E. Gervais

- Artist: **David E. Gervais**
- Source: the D. Gervais tiles library, <http://pousse.rapiere.free.fr/tome/> ("AngbandTk" download, `tiles/AngbandTk.zip`)
- Licence: [Creative Commons Attribution 3.0](http://creativecommons.org/licenses/by/3.0/) (CC BY 3.0)
- Changes: the spritesheets `dg_features32.gif`, `dg_grounds32.gif` and `dg_extra132.gif` were split into one 32×32 RGBA PNG per tile, named `<sheet>_<row>_<column>.png` (from 0, top left; empty tiles skipped), with the solid magenta (`#ff00ff`) background made transparent and names lower-cased. Only the tiles the game uses are here; the whole set is in `spare_assets/angbandtk/` (never shipped), which `tools/make_town_atlas.py` reads.
