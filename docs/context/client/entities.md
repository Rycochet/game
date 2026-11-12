# client: entities

Others on the map: mobs and other players, as tokens, for as long as the server says they can be seen. Also keeps the player's own token showing their kind of creature.

## Files

- `crates/client/src/entities/mod.rs`: `EntitiesPlugin`, `Others` (id to token), `read_server` (keeps whether the server's positions are in units, by the map messages in the order they come), `show_own_kind`, `picture`, `picture_size`. Unit tests.
- `crates/client/src/entities/moving.rs`: `Goal` (where another is going and how fast), `placed` (a position in the map's measure as a cell and an exact `SmoothPosition`), `stretch` (a `protocol::Stretch` as a `Goal` at its own speed), `show_moving`, `glide`.
- `crates/client/src/entities/morph.rs`: `Morphing` (a token changing from one body into another: the steps' pictures, then its new one), `Drawing::morph`, `play_morphs`, `SeenLooks` (how another creature was last seen to look).
- `crates/client/src/entities/mod.rs` also puts a `common::StaffMark` on another player seen with `EntityLook::mark`.
- `crates/client/src/entities/hover_name.rs`: `hover_name` (the `common::HoverName` each other gets: another player's character name from `EntityLook::name`, else the kind's).
- `crates/client/src/entities/dressed.rs`: `dressed` (an `IconPicture` of a kind wearing these in its own colours, from `item_art::character_look`), `show_own_dress` (the player's own token, from `inventory::Carried` and `character::CharacterSheet`, whenever what is worn or how they look changes).
- `crates/client/src/entities/pictures.rs`: `picture` (each kind's token picture), `picture_size`.
- `crates/client/src/entities/health_bars.rs`: `SeenHealth` (what the player's blows and Heals, and others' blows seen, left others, per mille, from `Landed::Hit`, `Landed::Healed` and `Harmed`), `take_health`, `refilled_share` (refilled here as Health refills, after `CALM_AFTER`, as nobody says again), `under_token` (a bar's place under a token, as wide as its base, by projecting it through `view3d::ViewCamera`), `show_health_bars` (`PostUpdate`, after transforms: a red bar under every token in sight whose owner isn't at full Health, the player's from `common::OwnVitals`; owner, 2026-11-07).
- `crates/client/src/entities/flash.rs`: `harmed` (who a message says was harmed: `AbilityLanded` with a `Hit` doing harm, at the player or their target, or `Harmed`), `flash_harmed` (puts `view3d::TokenFlash` on their token; owner, 2026-11-07).
- `crates/client/src/entities/wind_up_bars.rs`: `SeenWindUps` (others' wind-ups aimed at the player, from `AbilityStarted`), `take_wind_ups`, `show_wind_up_bars` (a gold bar filling under the health bar for any wind-up of a second or more: the player's from `common::OwnAbility`, or one aimed at them; attacks are quicker, so show only on the action bar), and for the player's own wait changing clothes when no wind-up is showing.
- `crates/client/src/entities/dressing_wait.rs` (placeholder, owner 2026-11-09, until a progress widget is designed): `DressingWait` (when the wait for changing clothes began and ends, from `common::Dressing` at the player's speed, changes while waiting adding on), `take_dressing`.

## Public API

- `EntitiesPlugin`.

## Depends on

- `common` (`HoverName`, `StaffMark`, `GameView`, `Icon`, `MapPosition`, `SmoothPosition`, `Token`, `TokenSize`), `connection` (`FromServer`), `inventory` (`Carried`), `character` (`CharacterSheet`), `player` (`Player`, `PlayerSystems`), `view3d` (`ClickedToken`, `View3dSystems`, `TokenFlash`), `item_art` (`ItemPictures`, `character_look`, `morph_look`, `MORPH_STEPS`), `shared::creature`, `shared::id`, `shared::protocol`.

## Used by

- Through `common::Other`: `inventory`.

## Notes

- `EnterLevel`, `OnWorldMap`, `Teleported` and `Welcome` forget everyone, as the server's `Client::changed_map` does, and it says again who is there.
- Others glide along the stretches the server sends (`ServerMessage::Moves`, in units on levels, timed in ticks) at each stretch's own speed (an ordinary walking pace when first seen), and jump if `TOO_FAR_BEHIND`. `MapPosition` is the cell they are in; `SmoothPosition` exactly where. They have the same tokens as anything else, so `view3d` and the views draw them and hide them where nobody can see.
- Every kind (all have a body layout) shows a picture of its body in its own colours in place of its tile (`IconPicture`), bare or dressed in whatever is drawn (armour, footwear): mobs and other players from `EntityLook::worn`, the player from their own inventory (owner's choice: the body is ungendered, so it looks right bare). Animals (horse, cattle, wolf, chicken) are drawn bare, and taurs (centaur, drider, scorpion man) dressed, and side on, in a square picture. A changed outfit is told as a new `EntitySeen`.
- Tiles (the Gervais set) are only seen until a kind's picture is drawn (a picture is blank until then, as `item_art` gives it); a centaur uses the human tile. A token's colour is how it feels about this player (`EntityLook::feeling`), told when it comes into sight and again when it changes. `common::Other` is on each token, for `inventory`'s Nearby panel.
- Others can't be clicked (dev tools act on the target instead: the debug menu's Examine Mob), but they can be hovered: `hover` names them. Other players never can be: their `feeling` is `None`.
- A change of kind (the player's own, or another seen again as another kind) is shown as a morph (owner, 2026-11-06): `MORPH_STEPS - 1` bare pictures of the body blending into the new one, all asked for at once, each shown for an eighth of a second once the first is drawn (any not drawn yet are skipped), then the new dressed picture once that is drawn. Worn things vanish for the change and come back on the new body. A change of outfit while it plays only changes what it ends on. If its pictures take more than 3 s, it gives up and shows the new picture.
