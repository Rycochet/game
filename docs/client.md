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
- **The minimap:** the round map in the top-right corner. Click it for the full map, which shows what you know of the world or of the site you're in.
- **The startup menu:** the game starts on its title with the system menu over it. Play joins the game: the last server joined with Connect..., or else single player for a native program and the page's server in a browser. If a server can't be played on, the game comes back here and says why.
- **Connect...:** only on the startup menu. Type a server's address (`host`, or `host:port`), or pick one joined before; the bin beside one forgets it. On a native program, the padlock takes the server's certificate or key fingerprint, pasted with Ctrl+V, and from then on only a server with that key is accepted.
- **Online (native only):** opens the game's own server to other players, once you press Play: they join with Connect... and your address. The gear beside it is Server Options: the server's name, the address players type, which of the computer's addresses and which port it listens on (8081 at first), a player limit, and its certificate with a copy button, for players who want to check it is you. Turning Online on or off while playing asks first, and players still connected are told the server is going offline. Online is remembered between runs.
- **The system menu:** Continue, Online and Connect... (greyed out while playing), then Settings and Credits, then Logout and Quit. Logout is only there when playing on a networked server, and goes back to the startup menu; the next Play is single player again, or the page's server in a browser. See below.
- **Interface size:** in Settings, under Interface. Auto (the first choice) keeps panels and writing the same share of the window however big it is; or choose from 50% to 300%. "Enable touch" keeps it at 100% or more, for fingers, and is always on for a device with only a touch screen. The minimap always keeps its share of the window.

## Pausing

The system menu opens with Esc or the gear button in the top-right corner. Whether the game pauses while it is open depends on the server:

| Playing on | While the system menu is open |
| ---------- | ------------------------------ |
| Single player (the server built into the native client) | The game pauses: the world's clock stops, and so does anything going by it. "Paused" shows in the menu. Not while Online is on, as others may be playing. |
| A networked server, and so every browser | Nothing pauses. The world carries on, the keys don't reach the game, and a journey you clicked on keeps going. |

So pausing needs both of these: a server that allows it, which only the built-in one does while it isn't online, and the system menu being open. A networked server never pauses, for anyone, whatever its settings. The game carries on as soon as the menu is closed.
