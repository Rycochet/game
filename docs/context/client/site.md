# client: site

Moving between maps: entering a world map poi's site, taking stairs between its levels, and leaving. The server decides all of it (it has the sites; the client doesn't): this module asks, then applies the answers. Drawing (stairs included) is `dungeon_map`.

## Files

- `crates/client/src/site/mod.rs`: `SitePlugin`. `follow_clicked_portal` (in `PlayerSystems::Input`, after `View3dSystems::Click`: a click on a `Portal`'s token sends `TravelTo` and remembers it in private `Heading`). In `PostUpdate`: `change_map` (reads `connection::FromServer`: `Welcome` keeps the world map position as the way back out; `EnterLevel` resets `CurrentDungeon`, puts the player on the arrival cell and switches to `GameView::Dungeon`; `LevelCells` go into `CurrentDungeon`, `RememberedLevel` into `OtherLevels`, which also keeps the level left by stairs and is emptied on `OnWorldMap`; `OnWorldMap` puts the player on the world map, in the copy of the planet they went in from (or are on); `Teleported` (dev tools) moves them on the current map, on the world map in the copy they are on), then `use_on_arrival` (on `player::Arrived`: at the poi heading for, sends `EnterSite` (after the steps there, so the server has the player at it), or at a World Gate `EnterGate`; in a site, on known stairs or on ground at the edge of the level (`shared::site::leads_out`), sends `UseExit`, unless `OwnVitals::fighting`). `EnterHall` puts the player in the Hall of Worlds (its `x` counted from `origin`, kept in `LevelKind::Hall { origin }` for the marble's hue and the gates' codes): `hall.rs` (`show_around`) makes `CurrentDungeon` a 300-tile stretch of it round them (`shared::hall::level`, all known and remembered, `LevelKind::Hall`), and `follow_player` (after `change_map`) moves the stretch along once they come within 75 tiles of its end (as far as the camera can see along it, `hall_view`); `GameView::Dungeon` draws it. `common::WorldReturn` keeps the world map position. `buttons.rs`: `down_or_up` (in `PlayerSystems::Input`: `Action::Down` (Shift + .) on the world map or in a site sends `Down`, `Action::Up` (Shift + ,) in the wilderness or any site sends `Up`, which the server checks; the same from the movement pad's Down and Up ladders (`step_pad`, found by `common::PadButton`; their `ButtonPressed`, sent once as one becomes held), enabled whenever they would work and disabled otherwise, their `HoverTip` saying what they do now or where they work: Down on the world map (Explore) or on stairs going down (Down), Up on stairs going up (Up) or, as Leave, on an outside cell of the wilderness or where `shared::site::can_leave` allows in a site (a Dev Area's key works anywhere, its button only by those rules); while `OwnVitals::fighting` both are `Disabled` with the tip "Not while fighting" and neither keys nor buttons send anything). The wilderness (`wilderness.rs`): `change_map` applies `WildernessWindow` (`CurrentDungeon::move_window`) and `InPoi` (`CurrentDungeon::in_poi`); `follow_point` (after `change_map`: in the wilderness, `WorldReturn` is the world map point the player is in, for the weather, the land's colours and leaving). Arriving on the window's edge ground doesn't send `UseExit` there.

## Public API

- `SitePlugin` only.

## Depends on

- `common` (`OtherLevels`, `CurrentDungeon`, `CurrentWorld`, `GameView`, `Portal`, `MapPosition`, `Preferences::controls`, `Action`), `widgets` (`ButtonState`, `HoverTip`, `ButtonPressed`), `common::PadButton` (the pad's ladders), `connection` (`ToServer`, `FromServer`), `player` (`Player`, `Arrived`, `TravelTo`, `PlayerSystems`), `view3d` (`ClickedToken`, `View3dSystems`), `shared::protocol`, `shared::site::{leads_out, can_leave}`, `shared::hall`.

## Used by

- Nothing directly (`dev_tools` enters sites by `ToServer(DevEnterSite)`).

## Notes

- Map changes run in `PostUpdate`, so the view being left never runs with the new position; the state changes next frame.
- Arriving means travel ended (`Arrived`), so keyboard steps onto stairs or ground don't use them, and passing over stairs on the way somewhere doesn't either. Clicking the cell the player stands on uses it at once (e.g. the stairs just arrived by).
- Nothing changes floors while fighting (owner, 2026-11-08): arriving on stairs then leaves the player standing on them, and Down or Up takes them once the fight is over. The server refuses too.
- The Up key (`<`) off stairs leaves from any outside cell of a site's first level, a cave's only from its edge (owner's rule, `can_leave`); the server keeps the cell in `left_at` as for walking out, so coming back in arrives there.
- Leaving by walking needs a cell on the very edge of the level (owner's rule): ground with walls or more ground on all four sides doesn't count, so a building's outside can be walked round to another door. The server checks it again.
- The way each staircase goes down is worked out by the server (`shared::site::Site::stairs_descend`) and arrives with its cell.
- What the player has seen of a level is remembered by the server with the character, and sent again (`remembered`) on coming back to it. On coming into a site (`LevelInfo::entering_site`) the server also sends what is remembered of its other levels.
