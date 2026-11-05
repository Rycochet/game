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
| Change Creature | Yes | Opens a panel with a button for each kind of creature. Choosing one makes your character that kind: everything comes off into the new inventory, and whatever doesn't fit is lost. |
| Cycle Temperature | Yes | Lowers the snow line all the way and raises it again, then turns itself off. Turning it off early puts the snow line straight back. |
| Detect Monsters | Yes | Toggles being shown every mob on the current level, whether you can see it or not. |
| Fields State | | Opens a panel stepping through the stages of the crops in view, on a level with fields. Changes only what is drawn. |
| Forget Map | Yes | Forgets what your character knows of this level, or on the world map, of the world and every site. Not in towns. |
| Forget Map (Server) | Yes | As Forget Map, and the server also forgets this site, or on the world map every site, so they are made again with the current generators. Works in towns. |
| Generate POI | Yes | Lists every kind of place. Choosing one opens a panel to pick its ruin and seed, then Generate takes you into it. |
| Give Item | Yes | Opens a panel that reads like what it makes: how many (up to a full stack, or 99 different ones), the size it is made for (`<target>` is your own; sizes are for what is worn, gems and blades, and everything else is always medium), whether they are different or identical, then its quality, whether it is part of a set, blessed or cursed, magical, and the kind of item (type to search). Only things worn can be blessed, cursed or magical, and only better than common can be magical, so asking for magic on something worse makes it uncommon. All are as secret as on anything found: use Truesight to see them. A seed (with Random beside it) decides everything else about it, and a picture shows what you will get. Give puts them in your inventory; in a site, what doesn't fit goes on the ground, and on the world map nothing is given unless all of it fits (it says No space). |
| Go to World | Yes | Opens a panel with a seed box (type one, or press Random) and Go, which takes you to that seed's world: back where you were on its map if you have been there, otherwise at one of its World Gates. Every number is a world. |
| God Mode | Yes | Toggles mobs taking no notice of you and letting you walk through them, so they don't block your way, and lets you take off cursed things. A ghost shows at the top left while it is on. |
| Herbs State | | Opens a panel stepping through how grown the herbs are, on a level with herbs. Changes only what is drawn. |
| Item Pictures | | Opens a panel showing one kind of item at a time, chosen from a list: a bare body of each kind on two legs, a sample of gauntlets, shoulder armour, arm armour, torso armour, helmets, waist armour, leg armour, foot armour or neck armour (in each style), armour sets of each kind (a full set in each style), footwear, weapons of each family, every kind of gem in every cut, or a sample of rings, as they are drawn, at each quality. It remembers the kind last chosen. Pressing a picture shows it large. |
| Kill Mob | Yes | Toggles clicking or tapping a mob in sight killing it. What it had falls where it stood, and it doesn't come back. A skull shows at the top left while it is on. |
| Reset View | | Puts the camera back to its starting view, as Home does. |
| Reveal Map | Yes | Shows you the whole of this level. Not in towns. |
| Show Debug Info | | Toggles the status text: position, hour, ping and view statistics. |
| Show Widgets | | Opens a panel with one of every piece of the interface. |
| Stairs Style | | Opens a panel stepping through the styles of stairs, on a level with stairs. Changes only what is drawn. |
| Suns and Moons | | Toggles a panel changing the colours of the suns and moons, and how bright the suns are. Only on your screen, until you next join the world. |
| Teleport Mode | Yes | Toggles clicks on the map taking you there at once, through walls in a site. |
| Teleport to Hall of Worlds | Yes | Takes you to the Hall of Worlds, a walkway hovering in the void, in front of the gate that leads to the World Gate nearest you. Teleport to Start brings you back. |
| Teleport to POI Showcase | Yes | Takes you to a plain far south with every kind of place on it, in several seeds and levels of ruin. |
| Teleport to Start | Yes | Takes you out of any site, to the nearest World Gate on the world map, where characters arrive. |
| Truesight | Yes | Toggles seeing what is hidden: the glow and names of items that are magical, blessed or cursed, and secret doors, which show as pale walls and stay shut. None count as found out, so turning it off hides them again. Eyes show at the top left while it is on. |
| View Planet | | Shows the whole planet as a turning globe, a day a minute, with clouds drifting over it and its moons going round it. The surface is coloured as on the world map, from pole to pole. The bar at the top shows another way of making the land (Generator), another planet (Seed, or the round arrow for a random one; the clipboard copies it) or heights, the plates, temperature or moisture instead of the surface (Layer); Reset goes back to this world. Pink rings mark the planet's World Gates, the ways onto it, and the bar says how they were found. Only the view changes, never the game. Any other click or tap, or Esc, closes it. |
| World Time | Yes | Toggles a panel moving the clock of the world you are in, for everyone in it: buttons step it from a minute to a day either way, and the box between them shows the time (such as `Day 2 06:30`) and moves the clock to whatever is typed there when you press Return or click away. Esc puts the real time back. It stops at the world's beginning. Below them, buttons from x1/4 to x600 set how fast that world's time runs from then on, the one in use picked out. |
