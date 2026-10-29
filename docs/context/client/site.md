# client: site

Moving between maps: entering a world map feature's site, taking stairs between its levels, and leaving. The server decides all of it (it has the sites; the client doesn't): this module asks, then applies the answers. Drawing (stairs included) is `dungeon_map`.

## Files

- `crates/client/src/site/mod.rs`: `SitePlugin`. `follow_clicked_portal` (in `PlayerSystems::Input`, after `View3dSystems::Click`: a click on a `Portal`'s token sends `TravelTo` and remembers it in private `Heading`). In `PostUpdate`: `change_map` (reads `connection::FromServer`: `EnterLevel` resets `CurrentDungeon`, puts the player on the arrival cell and switches to `GameView::Dungeon`; `LevelCells` go into `CurrentDungeon`; `LeftSite` puts the player back on the world map, in the copy of the planet they went in from), then `use_on_arrival` (on `player::Arrived`: at the feature heading for, sends `WorldMapPosition` (exactly there) and `EnterSite`; in a site, on known stairs or on ground at the edge of the level (`shared::site::leads_out`), sends `UseExit`). Private `WorldReturn` keeps the world map position.

## Public API

- `SitePlugin` only.

## Depends on

- `common` (`CurrentDungeon`, `CurrentWorld`, `GameView`, `Portal`, `MapPosition`), `connection` (`ToServer`, `FromServer`), `player` (`Player`, `Arrived`, `TravelTo`, `PlayerSystems`), `view3d` (`ClickedToken`, `View3dSystems`), `shared::protocol`, `shared::site::leads_out`.

## Used by

- Nothing directly (`dev_tools` enters sites by `ToServer(DevEnterSite)`).

## Notes

- Map changes run in `PostUpdate`, so the view being left never runs with the new position; the state changes next frame.
- Arriving means travel ended (`Arrived`), so keyboard steps onto stairs or ground don't use them, and passing over stairs on the way somewhere doesn't either. Clicking the cell the player stands on uses it at once (e.g. the stairs just arrived by).
- Leaving by ground needs a cell on the very edge of the level (owner's rule): ground with walls or more ground on all four sides doesn't count, so a building's outside can be walked round to another door. The server checks it again.
- The way each staircase goes down is worked out by the server (`shared::site::Site::stairs_descend`) and arrives with its cell.
- What the player has seen of a level is remembered by the server for the connection, and sent again (`remembered`) on coming back to it.
