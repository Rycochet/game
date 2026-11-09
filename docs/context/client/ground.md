# client: ground

Bags on the ground of a site, where the server says or the character remembers: a canvas sack by how full it looks, with a sparkle behind while in sight. What is in one is shown by `inventory`, which also has the Drop icon. Native and web.

## Files

- `crates/client/src/ground/mod.rs`: `GroundPlugin`, `Bags`, `Heading`, `BagAssets`, `make_assets`, `read_server`, `find_hovered`, `change_of_mind`, `follow_click`, `open_on_arrival`, `show_bags`, `GroundDebug` (native). Beside it: `parts.rs`, `assets.rs`, `spawn.rs`, `server.rs`, `clicks.rs` (the hover box from each bag's raised ground), `show.rs`, `raise.rs` (`raise_bags`: each bag on its cell's ground, `common::WildCellHeights`, when bags or heights change), `debug.rs` (`GroundDebug`, native only).
- `crates/client/src/ground/bag.rs`: without Bevy: `Model`, `bag`, `outline`, `star`, `size_of`, the shapes. Tests. In `bag/`: `model.rs`, `size.rs`, `shape.rs`, `outlines.rs`.

## Public API

- `GroundPlugin`, `GroundDebug` (message, native): `ShowLooks`, for `dev_tools`.

## Depends on

- `common` (`HoveredBag`, `ClickMode`, `MapPosition`, `PointerPosition`), `connection` (`FromServer`), `dungeon_map` (`CELL_SIZE`, `Shown`), `inventory` (`OpenCharacter`), `player` (`Player`, `Arrived`, `TakeStep`, `TravelTo`, `PlayerSystems`), `view3d` (`ClickedToken`, `View3dSystems`, `ViewCamera`, `XrayMaterial`, `XRAY_WHITE`), `shared::ground`, `shared::protocol`.

## Used by

- `dev_tools`. `dungeon_map` reads `common::HoveredBag`.

## Notes

- Owner's design. The look is only what the server last said (`GroundSeen`, `GroundGone`): out of sight it says nothing, so a remembered bag keeps its look, in grey with no sparkle, until seen again, though it may have gone. Whether a bag is in sight is `Shown::sight`, as for tokens.
- Fuller looks are drawn bigger, from half size for the flattest to full size for the most bulging (`Size::scale`). A sparkle is half size on the biggest bag and shrinks with the bag, while its outline keeps its width as drawn. Each look has `VARIETIES` shapes, and which one and how it is turned come from a hash of the cell, so bags side by side differ and a bag doesn't change as it fills unless its look does. Lit by real lights, as tokens are; grey ones are unlit.
- Seen through whatever hides it (`view3d`'s x-ray), in white, in sight or remembered: the bag's mesh as a faint fill and its outline mesh as the outline. Clicks reach it there too. Sparkles aren't.
- The sparkle is three stars facing the camera, behind the bag along the ground, so it shows above it. It goes by real time and is unlit, to be seen in the dark. Bag and stars always have a black outline (inverted hull; larger black stars behind them), wider while hovered, with the same check on screen for mouse and finger.
- Clicking a bag walks there along a found route and opens the Character panel on `Arrived`, where `inventory` shows the ground section. A step of the player's own or a journey elsewhere drops that, and keyboard movement never opens it. Other click modes (teleporting) ignore bags. By touch, a tap arrives as a click, and there is no hover but while the finger is down.
