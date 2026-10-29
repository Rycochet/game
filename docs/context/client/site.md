# client: site

Moving between maps: entering a world map feature's site, taking stairs between its levels, and leaving. Generation is `shared::site`; drawing (stairs included) is `dungeon_map`.

## Files

- `crates/client/src/site/mod.rs`: `SitePlugin`. `follow_clicked_portal` (in `PlayerSystems::Input`, after `View3dSystems::Click`: a click on a `Portal`'s token sends `TravelTo` and remembers it in private `Heading`). In `PostUpdate`: `use_on_arrival` (on `player::Arrived`: at the feature heading for, enter it; in a site, on a stairs cell take them, and on outside ground at the edge of the level (`shared::site::leads_out`) leave for the world map), then `change_map` (handles `EnterFeature` then `TakeExit`: generates the level into `CurrentDungeon` (level, stairs), puts the player where they arrive, switches `GameView`). `descends` / `descent_way`: the way each staircase goes down, from both levels it joins (the other is generated again to look): floor to step on at the top and off at the bottom, then a wall at each far end. Unit test. Private `WorldReturn` keeps the world map position.

## Public API

- `EnterFeature(Feature)` (message): enter that feature's site (level 0, at its entrance).
- `TakeExit(Exit)` (message): go to another level (arriving on the stairs back) or the world map (where the player entered from).

## Depends on

- `common` (`CurrentDungeon`, `LevelStairs`, `GameView`, `Portal`, `MapPosition`), `player` (`Player`, `Arrived`, `TravelTo`, `PlayerSystems`), `view3d` (`ClickedToken`, `View3dSystems`), `shared::site`.

## Used by

- `dev_tools` (`GAME_ENTER` sends both messages).

## Notes

- Map changes run in `PostUpdate`, so the view being left never runs with the new position; the state changes next frame.
- Arriving means travel ended (`Arrived`), so keyboard steps onto stairs or ground don't use them, and passing over stairs on the way somewhere doesn't either. Clicking the cell the player stands on uses it at once (e.g. the stairs just arrived by).
- Leaving by ground needs a cell on the very edge of the level (owner's rule): ground with walls or more ground on all four sides doesn't count, so a building's outside can be walked round to another door.
- Stairs are cells of the level (`CurrentDungeon::stairs`), not entities. Their way only matters for looks, so it is worked out here, not in `shared`; both ends of a staircase get the same arguments, so they always agree (going down westwards from the top means climbing westwards from the bottom).
- Levels are regenerated on every visit (cheap); nothing is remembered between visits yet.
