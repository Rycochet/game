---
title: Debug menu
nav_order: 7
---

# Debug menu

The debug menu holds tools for trying things out while the game is being made. Open it with backtick (`` ` ``) or the warning button at the top left. The entries are in groups, each under its heading. Type to filter them, then choose an entry with a click, a tap, or the arrow keys and Return (Left and Right move along what you typed until Up or Down picks an entry, then move to the nearest entry in the next column). An entry ending in "..." opens a panel, a list or a view; the others act at once, or turn something on or off.

Each entry is a permission of its own, and you only see those you may use; with none (and no Admin Tools), there is no debug menu at all, nor its button, key or row in Controls. Single player gives its own player all of them. A networked server gives each to the [roles](hosting/#roles) that have it: by default Owners and Admins have all of them, Developers all but World Time and Freeze Mobs, and Moderators those for looking after players. With `HOW_DEBUG_TOOLS` set to `true` (see [Server environment variables](environment-variables)), everyone connected has all but World Time and Freeze Mobs. Buttons inside a panel that do something of their own, such as Examine Mob's Kill, are hidden too without their permission. An entry that can't be used right now stays in the list, greyed out. Entries marked **Server** ask the server to do something, which it checks.

## Display

What only your screen shows.

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| Show Debug Info |  | Toggles the hour and the view statistics in the status text at the top left. Your position, the frame rate (🎬, frames a second over the last two seconds) and the ping always show there. |
| Lightning Now | | Flashes lightning where you are, as in the heaviest rain: everything on the world map, or only what is outside on a site's level, with more light spilling in through doors and windows. Only you see it, and without a bolt. Dimmer and slower with Reduce Motion. |
| Suns and Moons | | Toggles a panel changing the colours of the suns and moons, and how bright the suns are. Only on your screen, until you next join the world. |
| Drawing States |  | Opens a panel with a row each for stairs, fields and herbs, whose buttons step through the styles of stairs, the stages of the crops and how grown the herbs are on the level you are on. A row is greyed where the level has none. Changes only what is drawn. |

## Galleries

Pictures of what the game draws. They need no server.

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| Item Pictures | | Opens a panel showing one kind of item at a time, chosen from a list: a bare body of each kind on two legs and of each animal drawn, a sample of gauntlets, shoulder armour, arm armour, torso armour, helmets, waist armour, leg armour, foot armour or neck armour (in each style), armour sets of each kind (a full set in each style), footwear, underclothes, weapons of each family, every kind of gem in every cut, or a sample of rings, as they are drawn, at each quality. It remembers the kind last chosen. Pressing a picture shows it large. |
| Show Widgets | | Opens a panel with one of every piece of the interface. Drop an item from the inventory on its text box to write it in. At the bottom is a sample of the interface written as markup. |
| View Planet | | Shows the whole planet as a turning globe, a day a minute, with clouds drifting over it and its moons going round it. The surface is coloured as on the world map, from pole to pole. The bar at the top shows another way of making the land (Generator), another planet (Seed, or the round arrow for a random one; the clipboard copies it) or heights, the plates, temperature or moisture instead of the surface (Layer); Reset goes back to this world. Pink rings mark the planet's World Gates, the ways onto it, and the bar says how they were found. Only the view changes, never the game. Drag with the right mouse button (or two fingers) to turn and tilt the globe, and use the wheel (or pinch) to zoom; the camera keys and Home work too. Pointing at a ring names its gate. Click or tap a ring to go to that gate, or press **Teleport to World...** to open that panel with this planet's seed in it; both need the **Server**, and work only on the planet it opened for, with its own generator. Any other click or tap, or Esc, closes it. |

## Travel

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| Teleport Mode | Yes | Toggles clicks on the map taking you there at once, through walls in a site. A click on a place or a World Gate takes you there and in. |
| Teleport to World | Yes | Opens a panel with a seed box (type one, press the new seed button, or leave it blank for the world you are on, or in the Hall of Worlds the one you came from), a row of gates (**Any**, or World Gates 1 to 16) and three buttons. **Go** takes you to that world: to the gate chosen, or with **Any** back where you were on its map if you have been there, otherwise at one of its World Gates. **Hall of Worlds** takes you to the Hall, a walkway hovering in the void, in front of the gate that leads to the gate chosen (with **Any**, the World Gate nearest you). **View Planet...** shows that world as a globe. Every number is a world. |

## Character

Your own character.

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| God Mode | Yes | Toggles mobs taking no notice of you and letting you walk through them, so they don't block your way, and lets you take off cursed things. A ghost shows at the top left while it is on. |
| Truesight | Yes | Toggles seeing what is hidden: the glow and names of items that are magical, blessed or cursed, and secret doors, which show as walls with the outline of a door and stay shut. None count as found out, so turning it off hides them again. Eyes show at the top left while it is on. |
| Change Creature | Yes | Opens a panel with a button for each kind of creature, and three for how the change is made. **Permanent** makes your character that kind for good. **Temporary** changes only your body: you take on its strength, agility and toughness, but keep your own mind and spirit, and changing back to your own kind gives you back exactly what you had. **Illusion** is not made yet. Your token shows the change as it happens. Whatever the new body can still wear stays on, and what it can't, or has no room to carry, drops to the ground where you stand. Not on the world map or in the Hall of Worlds, which have no ground: step into the wilderness first. |
| Give Item | Yes | Opens a panel that reads like what it makes: how many (up to a full stack, or 99 different ones), the size it is made for (`<target>` is your target's own, or yours with no target in sight; sizes are for what is worn, gems and blades, and everything else is always medium), whether they are different or identical, then its quality, whether it is part of a set, blessed or cursed, magical, and the kind of item (type to search). Only things worn can be blessed, cursed or magical, and only better than common can be magical, so asking for magic on something worse makes it uncommon. All are as secret as on anything found: use Truesight to see them. For kinds whose make can be chosen, a Form line offers what each choice can be, given the others: style, construction, material, finish and colour for armour, material and colour for footwear and underclothes, metal, settings, gem and cut for rings, gem and cut for gems, metal and length for blades; anything left at Any is rolled. A seed (with a new seed button beside it) decides everything else about it, and a picture shows what you will get. Give gives them to your target while it is in sight (beside Give it says who: to you, or to the wolf, say), made to fit them: a creature keeps whatever it is given, and another player gets them in their inventory as you would. With no target in sight, Give puts them in your inventory; in a site, what doesn't fit goes on the ground, and on the world map nothing is given unless all of it fits (it says No space). If what is chosen can't be made from that seed, it says so and gives nothing. |
| Vitals | Yes | Opens a panel with four buttons: **Hurt 5** takes 5 Health, as a blow would (refilling waits 10 seconds, and it counts towards raising your most Health); **Spend 5 Mana** spends 5 Mana if you have it, as casting would; **Heal Fully** fills your Health and Mana; **Bonus** raises your most Health and Mana by 5 for 5 minutes. |
| Learn Skills | Yes | Opens a panel with every skill as a checkbox, set in under the skill above it, and a box beside each with its value as a whole percentage. **For** picks whose skills they are: **You**, or **Target**, another player you have targeted who is in sight (creatures have no skills yet). Ticking a skill learns it at 10%, and the skills above it their share; unticking forgets it and everything under it. Typing a value in a known skill's box and leaving the box sets that skill alone to it. At the top, **Random** practises six skills at the ends of branches by random amounts, **Clear Skills** forgets every skill, and **Set All** sets every skill to the value in the box beside it, to try what a skill at that value does. |

## Mobs

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| Create Mob | Yes | Opens a panel for a mob made to order: its kind, how it feels about you, whether it has its kind's gear, whether it stands Still (doing nothing, even when hit) and a seed for its looks. Create turns Mob Mode on, shown in red at the top: each click then puts that mob on the cell clicked, instead of walking there. Press Create again, or close the panel, to stop. Only in sites, not the wilderness or the Hall, and the mobs are not kept, though the panel's choices are, between runs. |
| Examine Mob | Yes | Opens a panel showing what the server knows of the creature you have targeted (right-click or long press it): what it is doing, its target, Health, attributes and how it feels about you, kept up to date while the panel is open. **Kill** kills it, leaving what it had where it stood (it doesn't come back, and counts as a death, so your target moves to the nearest enemy you can see); **Destroy** takes it away for good, leaving nothing; **Give Item...** opens Give Item, which gives to your target. |
| Freeze Mobs | Yes | Toggles every mob in the world standing still and doing nothing; they can still be hit. |
| Detect Mobs | Yes | Toggles being shown every mob on the current level, whether you can see it or not. |

## Level

The level or map you are on.

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| Reveal Map | Yes | Shows you the whole of this level. Not in towns. |
| Forget Map | Yes | Forgets what your character knows of this level, or on the world map, of the world and every site. Not in towns. |
| Remake Site | Yes | Forgets what your character knows of this level, as Forget Map does, and the server also forgets this site, or on the world map every site, so they are made again with the current generators. Works in towns. |
| Change Cell Mode | Yes | Toggles clicks in a site changing the cell clicked, from floor to wall to door to window and back, to try wall shapes. |
| Open All Doors | Yes | Opens every closed door on the level you are on, away from you, secret doors too: one you see open is found. |
| Close All Doors | Yes | Closes every open door on the level you are on. Broken doors stay open, and so does any door with someone or something in its doorway. |
| Key for This Door | Yes | Gives you the key to the lock of every door beside you, or if there is none, to the nearest lock on the level you are on. |

## World

The whole world, and making places to try things in.

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| World Time | Yes | Toggles a panel moving the clock of the world you are in, for everyone in it: buttons step it from a minute to a day either way, and the box between them shows the time (such as `Day 2 06:30`) and moves the clock to whatever is typed there when you press Return or click away. Esc puts the real time back. It stops at the world's beginning. Below them, buttons from x1/4 to x600 set how fast that world's time runs from then on, the one in use picked out. |
| Lightning Strike Here | Yes | Lightning strikes near you now, whatever the weather: a flash for you and a bolt close by (never on anyone, and not by a poi), which others nearby see too. Bolts show on the world map and in the wilderness; in other places only the flash does. |
| Generate POI | Yes | Lists every kind of place. Choosing one opens a panel to pick its ruin and seed, then Generate takes you into it. |
| Dev Area | Yes | Opens a panel making a walled area to try things in: its size (16 to 128 cells a side), what fills it (nothing, a moat you can see across, a wall up the middle, the wild land of any biome, or the ground floor of any kind of place, cut off where it is bigger), who is in it (whoever would live there, nobody, or a kind of creature and how many) and a seed of any words. Generate puts you on its stairs; going up them, or Shift + , (`<`) anywhere in it, takes you back to where you were. It is never kept, but the panel's choices are, between runs. |
| Regenerate Dev Area | Yes | Makes the Dev Area its panel was last set to again, as new, and puts you in it, without opening the panel. |

## Admin Tools

Looking after the server and its players. Unlike the others, this entry goes by your [role](hosting/#roles) rather than a permission of its own: you see it when your role lets you look players up, give roles, rename them or read the audit log, even with no other debug tool.

| Entry | Server | What it does |
| ----- | ------ | ------------ |
| Accounts | Yes | Opens the Accounts panel: **Find** looks a character up by name and gives their account another role, **Roles** renames roles, and **Audit Log** shows what has been changed. See [Roles](hosting/#roles). |
