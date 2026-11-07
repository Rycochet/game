---
title: The client
nav_order: 3
---

# The client

The client is the game itself. It looks much the same as a native program and in a browser, but the two differ in what they connect to:

- **Native** programs play single player on a server built into them, or join a networked server chosen with Connect... on the startup menu.
- **Browsers** play on the server the page came from, or another chosen with Connect... (one whose certificate is from a certificate authority, as browsers check that themselves).

In every case the server decides what happens, and the client shows it.

## What you see

- **The world map:** the land in 3D, with its rivers, weather, and suns and moons moving across the sky. Time passes for everyone on a server, and the places you can visit stand on the map.
- **Sites:** go into a place on the map and the view changes to its levels, seen cell by cell. You only see what your character can see, and what you have seen before stays on the map in grey.
- **Health and Mana:** the circle at the bottom middle, Health on the left in red and Mana on the right in blue, each filling from the bottom. Hold the pointer or a finger on it to see the numbers. Ten seconds after you are last hurt or spend Mana, both refill, empty to full in about five minutes. The Character panel shows where the most of each comes from: an attribute, playing, practice and anything raising them for now.
- **The minimap:** the round map in the top-right corner. Click it for the full map, which shows what you know of the world or of the site you're in.
- **The startup menu:** the game starts on its title with the system menu over it. Play joins the game: the last server joined with Connect..., or else single player for a native program and the page's server in a browser. If a server can't be played on, the game comes back here and says why.
- **Connect...:** only on the startup menu. Type a server's address (`host`, or `host:port`), or pick one joined before; the bin beside one forgets it. Until you have joined one, the box suggests the game's own server, `hallofworlds.app`. Once you stop typing, the address is checked: a spinner, then a green tick when a Hall of Worlds server answers, or a red cross saying why not (hover over it). Without a port, port 24691 is tried and then 443, and the one that answered is added to the address. Connect only works after a tick. On a native program, the padlock takes the server's certificate or key fingerprint, pasted with Ctrl+V, and from then on only a server with that key is accepted.
- **Online (native only):** opens the game's own server to other players, once you press Play: they join with Connect... and your address. The gear beside it is Server Options: the server's name, the address players type, which of the computer's addresses and which port it listens on (24691 at first), a player limit, a switch for asking your router to forward that port (UPnP, on at first) with a line saying how it went and the address players on the internet use, and its certificate with a copy button, for players who want to check it is you. Turning Online on or off while playing asks first, and players still connected are told the server is going offline. Online is remembered between runs.
- **The system menu:** Continue, Online and Connect... (greyed out while playing), then Settings and Credits, then Logout and Quit. Logout is only there when playing on a networked server, and goes back to the startup menu; the next Play is single player again, or the page's server in a browser. See below.
- **Interface size:** in Settings, under Interface. Auto (the first choice) keeps panels and writing the same share of the window however big it is; or choose from 50% to 300%. "Enable Touch" keeps it at 100% or more, for fingers, and is always on for a device with only a touch screen. The minimap always keeps its share of the window.
- **Video (native only):** in Settings, under Video. Display mode is Windowed, Borderless full screen (a window without borders filling the screen) or Full screen (the screen taken over at the resolution chosen below it, its own at first). Nothing changes until Apply, which asks whether to keep the new settings and goes back to the old ones after 15 seconds if not, in case the screen can't show them. Alt+Enter switches between windowed and the last full screen mode. The window opens as it was left: where it was, its size, maximised or not, on the same monitor if that is still there. In a browser these are shown but can't be changed, as the browser has the window. Below them, settings that trade looks for speed or clutter apply at once: a Preset (Low, Medium or High, or Custom once any is changed), then Frame Rate Limit, VSync, Render Scale (the world drawn at a lower resolution, the interface still sharp) and Anti-aliasing; Lighting (Brightness, which lifts the darkest parts of what you can see to just under half of daylight, Shadows, and Midday Sun, the world map always lit as at noon); World Map (View Distance, Clouds, Weather and Foliage; clouds shade the ground only with Shadows on too); Sites (Floor Textures, or plain colours, and how Remembered Places are drawn); Motion (Reduce Motion: still water, and tokens and the camera jumping rather than gliding); and what the game does while another window is in front (slow down to a frame a second, or keep drawing). Presets never change Brightness or Reduce Motion. Hold the pointer on any setting to see what it does. Those the browser decides are shown, greyed out.

## Pausing

The system menu opens with Esc or the gear button in the top-right corner. Whether the game pauses while it is open depends on the server:

| Playing on | While the system menu is open |
| ---------- | ------------------------------ |
| Single player (the server built into the native client) | The game pauses: the server's clock stops, and every world's time with it, and so does anything going by them. "Paused" shows in the menu. Not while Online is on, as others may be playing. |
| A networked server, and so every browser | Nothing pauses. The world carries on, the keys don't reach the game, and a journey you clicked on keeps going. |

Playtime only counts while you are doing something: it stops 5 minutes after you last did anything. On a networked server, or as a guest on a game that is Online, doing nothing for 15 minutes disconnects you (the server may set another time). Single player never disconnects you.

So pausing needs both of these: a server that allows it, which only the built-in one does while it isn't online, and the system menu being open (or, with "Pause When Not in Focus" on under Settings, Gameplay, another window being in front). A networked server never pauses, for anyone, whatever its settings. The game carries on as soon as the menu is closed.
