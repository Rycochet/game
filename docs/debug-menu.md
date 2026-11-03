---
title: Debug menu
nav_order: 7
---

# Debug menu

The debug menu holds tools for trying things out while the game is being made. Open it with backtick (`` ` ``) or the warning button at the top left. Type to filter the list, then choose an entry with a click, a tap, or the arrow keys and Return.

An entry that can't be used right now stays in the list, greyed out. Entries marked **Server** also need the server to allow them. Single player always does, and a networked server only with `HOW_DEBUG_TOOLS` set to `true` (see [Server environment variables](environment-variables)), which then allows them for everyone connected.

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| Auto Walk | | Toggles walking east, one step a frame, on the world map. |
| Change Cell Mode | Yes | Toggles clicks in a site changing the cell clicked, from floor to wall to door to window and back, to try wall shapes. |
| Change Creature | Yes | Opens a panel with a button for each kind of creature. Choosing one makes your character that kind: everything comes off into the new pockets, and whatever doesn't fit is lost. |
| Cycle Temperature | Yes | Lowers the snow line all the way and raises it again, then turns itself off. Turning it off early puts the snow line straight back. |
| Detect Monsters | Yes | Toggles being shown every mob on the current level, whether you can see it or not. |
| Fields State | | Opens a panel stepping through the stages of the crops in view, on a level with fields. Changes only what is drawn. |
| Forget Map | Yes | Forgets what your character knows of this level, or on the world map, of the world and every site. Not in towns. |
| Forget Map (Server) | Yes | As Forget Map, and the server also forgets this site, or on the world map every site, so they are made again with the current generators. Works in towns. |
| Generate POI | Yes | Lists every kind of place. Choosing one opens a panel to pick its ruin and seed, then Generate takes you into it. |
| Give Item | Yes | Opens a panel with a button for each kind of item, and choices of the size it is made for (for what is worn, gems and blades; everything else is always medium), whether it is magical, whether it is blessed or cursed, whether it is part of a set and its quality. Only things worn can be any of the first three, and only better than common can be magical, so asking for magic on something worse makes it uncommon. All are as secret as on anything found: use Truesight to see them. Each press puts a new one in your pockets, if there is room. |
| God Mode | Yes | Toggles mobs taking no notice of you and letting you walk through them, so they don't block your way, and lets you take off cursed things. A ghost shows at the top left while it is on. |
| Herbs State | | Opens a panel stepping through how grown the herbs are, on a level with herbs. Changes only what is drawn. |
| Item Pictures | | Opens a panel showing one kind of item at a time, chosen from a list: a bare body of each kind on two legs, a sample of gauntlets, shoulder armour, arm armour, torso armour or helmets (in each style), weapons of each family, every kind of gem in every cut, or a sample of rings, as they are drawn, at each quality. It remembers the kind last chosen. Pressing a picture shows it large. |
| Kill Mob | Yes | Toggles clicking or tapping a mob in sight killing it. What it had falls where it stood, and it doesn't come back. A skull shows at the top left while it is on. |
| Reset View | | Puts the camera back to its starting view, as Home does. |
| Reveal Map | Yes | Shows you the whole of this level. Not in towns. |
| Show Debug Info | | Toggles the status text: position, hour, ping and view statistics. |
| Show Widgets | | Opens a panel with one of every piece of the interface. |
| Stairs Style | | Opens a panel stepping through the styles of stairs, on a level with stairs. Changes only what is drawn. |
| Suns and Moons | | Toggles a panel changing the colours of the suns and moons, and how bright the suns are. Only on your screen, until you next join the world. |
| Teleport Mode | Yes | Toggles clicks on the map taking you there at once, through walls in a site. |
| Teleport to POI Showcase | Yes | Takes you to a plain far south with every kind of place on it, in several seeds and levels of ruin. |
| Teleport to Start | Yes | Takes you out of any site, to where new characters start on the world map. |
| Truesight | Yes | Toggles seeing what is hidden: the glow and names of items that are magical, blessed or cursed, and secret doors, which show as pale walls and stay shut. None count as found out, so turning it off hides them again. Eyes show at the top left while it is on. |
| World Time | Yes | Toggles a panel moving the world's clock an hour either way, for everyone playing. It stops at the world's beginning. |
