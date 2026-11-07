---
title: Debug menu
nav_order: 7
---

# Debug menu

The debug menu holds tools for trying things out while the game is being made. Open it with backtick (`` ` ``) or the warning button at the top left. Type to filter the list, then choose an entry with a click, a tap, or the arrow keys and Return (Left and Right move along what you typed until Up or Down picks an entry).

An entry that can't be used right now stays in the list, greyed out. Entries marked **Server** also need the server to allow them. Single player always does, and a networked server only with `HOW_DEBUG_TOOLS` set to `true` (see [Server environment variables](environment-variables)), which then allows them for everyone connected.

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| Auto Walk | | Toggles walking east, one step a frame, on the world map. |
| Change Cell Mode | Yes | Toggles clicks in a site changing the cell clicked, from floor to wall to door to window and back, to try wall shapes. |
| Change Creature | Yes | Opens a panel with a button for each kind of creature, and three for how the change is made. **Permanent** makes your character that kind for good. **Temporary** changes only your body: you take on its strength, agility and toughness, but keep your own mind and spirit, and changing back to your own kind gives you back exactly what you had. **Illusion** is not made yet. Your token shows the change as it happens. Whatever the new body can still wear stays on, and what it can't, or has no room to carry, drops to the ground where you stand. Not on the world map or in the Hall of Worlds, which have no ground: step into the wilderness first. |
| Delete Character | Yes | Asks first, then deletes your character, everything they carry and know included, and opens the screen for making a new one. Until characters can be chosen, this is how to make another. |
| Detect Monsters | Yes | Toggles being shown every mob on the current level, whether you can see it or not. |
| Fields State | | Opens a panel stepping through the stages of the crops in view, on a level with fields. Changes only what is drawn. |
| Forget Map | Yes | Forgets what your character knows of this level, or on the world map, of the world and every site. Not in towns. |
| Forget Map (Server) | Yes | As Forget Map, and the server also forgets this site, or on the world map every site, so they are made again with the current generators. Works in towns. |
| Forget Skills | Yes | Forgets every skill but those your kind starts with. |
| Generate POI | Yes | Lists every kind of place. Choosing one opens a panel to pick its ruin and seed, then Generate takes you into it. |
| Give Item | Yes | Opens a panel that reads like what it makes: how many (up to a full stack, or 99 different ones), the size it is made for (`<target>` is your target's own, or yours with no target in sight; sizes are for what is worn, gems and blades, and everything else is always medium), whether they are different or identical, then its quality, whether it is part of a set, blessed or cursed, magical, and the kind of item (type to search). Only things worn can be blessed, cursed or magical, and only better than common can be magical, so asking for magic on something worse makes it uncommon. All are as secret as on anything found: use Truesight to see them. For kinds whose make can be chosen, a Form line offers what each choice can be, given the others: style, construction, material, finish and colour for armour, material and colour for footwear, metal, settings, gem and cut for rings, gem and cut for gems, metal and length for blades; anything left at Any is rolled. A seed (with a new seed button beside it) decides everything else about it, and a picture shows what you will get. Give gives them to your target while it is in sight (beside Give it says who: to you, or to the wolf, say), made to fit them: a creature keeps whatever it is given, and another player gets them in their inventory as you would. With no target in sight, Give puts them in your inventory; in a site, what doesn't fit goes on the ground, and on the world map nothing is given unless all of it fits (it says No space). If what is chosen can't be made from that seed, it says so and gives nothing. |
| Go to World | Yes | Opens a panel with a seed box (type one, or press the new seed button) and Go, which takes you to that seed's world: back where you were on its map if you have been there, otherwise at one of its World Gates. Every number is a world. |
| God Mode | Yes | Toggles mobs taking no notice of you and letting you walk through them, so they don't block your way, and lets you take off cursed things. A ghost shows at the top left while it is on. |
| Heal Fully | Yes | Fills your Health and Mana. |
| Herbs State | | Opens a panel stepping through how grown the herbs are, on a level with herbs. Changes only what is drawn. |
| Hurt | Yes | Takes 5 Health, as a blow would: refilling waits 10 seconds, and it counts towards raising your most Health. |
| Item Pictures | | Opens a panel showing one kind of item at a time, chosen from a list: a bare body of each kind on two legs and of each animal drawn, a sample of gauntlets, shoulder armour, arm armour, torso armour, helmets, waist armour, leg armour, foot armour or neck armour (in each style), armour sets of each kind (a full set in each style), footwear, weapons of each family, every kind of gem in every cut, or a sample of rings, as they are drawn, at each quality. It remembers the kind last chosen. Pressing a picture shows it large. |
| Kill Mob | Yes | Toggles clicking or tapping a mob in sight killing it. What it had falls where it stood, and it doesn't come back. A skull shows at the top left while it is on. It counts as a death, so if it was your target, the nearest enemy you can see becomes your target. |
| Learn Some Skills | Yes | Practises six skills at the ends of branches, picked at random, by random amounts, and so the skills above them. |
| Reset View | | Puts the camera back to its starting view, as Home does. |
| Reveal Map | Yes | Shows you the whole of this level. Not in towns. |
| Show Debug Info | | Toggles the status text: position, hour, ping and view statistics. |
| Show Widgets | | Opens a panel with one of every piece of the interface. Drop an item from the inventory on its text box to write it in. |
| Spend Mana | Yes | Spends 5 Mana, if you have that much, as casting would: it counts towards raising your most Mana. |
| Stairs Style | | Opens a panel stepping through the styles of stairs, on a level with stairs. Changes only what is drawn. |
| Suns and Moons | | Toggles a panel changing the colours of the suns and moons, and how bright the suns are. Only on your screen, until you next join the world. |
| Teleport Mode | Yes | Toggles clicks on the map taking you there at once, through walls in a site. |
| Teleport to Hall of Worlds | Yes | Takes you to the Hall of Worlds, a walkway hovering in the void, in front of the gate that leads to the World Gate nearest you. Teleport to Start brings you back. |
| Teleport to POI Showcase | Yes | Takes you to a plain far south with every kind of place on it, in several seeds and levels of ruin. |
| Teleport to Start | Yes | Takes you out of any site, to the nearest World Gate on the world map, where characters arrive. |
| Truesight | Yes | Toggles seeing what is hidden: the glow and names of items that are magical, blessed or cursed, and secret doors, which show as pale walls and stay shut. None count as found out, so turning it off hides them again. Eyes show at the top left while it is on. |
| View Planet | | Shows the whole planet as a turning globe, a day a minute, with clouds drifting over it and its moons going round it. The surface is coloured as on the world map, from pole to pole. The bar at the top shows another way of making the land (Generator), another planet (Seed, or the round arrow for a random one; the clipboard copies it) or heights, the plates, temperature or moisture instead of the surface (Layer); Reset goes back to this world. Pink rings mark the planet's World Gates, the ways onto it, and the bar says how they were found. Only the view changes, never the game. Any other click or tap, or Esc, closes it. |
| Vitals Bonus | Yes | Raises your most Health and Mana by 5 for 5 minutes. |
| World Time | Yes | Toggles a panel moving the clock of the world you are in, for everyone in it: buttons step it from a minute to a day either way, and the box between them shows the time (such as `Day 2 06:30`) and moves the clock to whatever is typed there when you press Return or click away. Esc puts the real time back. It stops at the world's beginning. Below them, buttons from x1/4 to x600 set how fast that world's time runs from then on, the one in use picked out. |
