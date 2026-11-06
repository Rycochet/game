# client: entities

Others on the map: mobs and other players, as tokens, for as long as the server says they can be seen. Also keeps the player's own token showing their kind of creature.

## Files

- `crates/client/src/entities/mod.rs`: `EntitiesPlugin`, `Others` (id to token), `read_server`, `show_moving`, `show_own_kind`, `offer_to_kill`, `kill_clicked`, `picture`, `picture_size`. Unit tests.
- `crates/client/src/entities/dressed.rs`: `dressed` (an `IconPicture` of a kind wearing these in its own colours, from `item_art::character_look`), `show_own_dress` (the player's own token, from `inventory::Carried` and `character::CharacterSheet`, whenever what is worn or how they look changes).

## Public API

- `EntitiesPlugin`.

## Depends on

- `common` (`Clickable`, `GameView`, `Icon`, `KillMob`, `MapPosition`, `SmoothPosition`, `Token`, `TokenSize`), `connection` (`FromServer`), `inventory` (`Carried`), `character` (`CharacterSheet`), `player` (`Player`, `PlayerSystems`), `view3d` (`ClickedToken`, `View3dSystems`), `item_art` (`ItemPictures`, `character_look`), `shared::creature`, `shared::id`, `shared::protocol`.

## Used by

- Through `common::Other`: `inventory`.

## Notes

- `EnterLevel`, `OnWorldMap`, `Teleported` and `Welcome` forget everyone, as the server's `Client::changed_map` does, and it says again who is there.
- Others glide towards where the server says at an ordinary walking pace, and jump if `TOO_FAR_BEHIND`. They have the same tokens as anything else, so `view3d` and the views draw them and hide them where nobody can see.
- Every kind (all have a body layout) shows a picture of its body in its own colours in place of its tile (`IconPicture`), bare or dressed in whatever is drawn (armour, footwear): mobs and other players from `EntityLook::worn`, the player from their own inventory (owner's choice: the body is ungendered, so it looks right bare). Animals (horse, cattle, wolf, chicken) are drawn bare, and taurs (centaur, drider, scorpion man) dressed, and side on, in a square picture. A changed outfit is told as a new `EntitySeen`.
- Tiles (the Gervais set) are only seen until a kind's picture is drawn (a picture is blank until then, as `item_art` gives it); a centaur uses the human tile. A token's colour is how it feels about this player (`EntityLook::feeling`), told when it comes into sight and again when it changes. `common::Other` is on each token, for `inventory`'s Nearby panel.
- Others can't be clicked, except mobs while the debug menu's Kill Mob is on. Other players never can be: their `feeling` is `None`.
