---
title: The client
nav_order: 3
---

# The client

The client is the game itself. It looks much the same as a native program and in a browser, but the two differ in what they connect to:

- **Native** programs play single player on a server built into them, or join a networked server if told to.
- **Browsers** always play on the server the page came from.

In every case the server decides what happens, and the client shows it.

## What you see

- **The world map:** the land in 3D, with its rivers, weather, and suns and moons moving across the sky. Time passes for everyone on a server, and the places you can visit stand on the map.
- **Sites:** go into a place on the map and the view changes to its levels, seen cell by cell. You only see what your character can see, and what you have seen before stays on the map in grey.
- **The minimap:** the round map in the top-right corner. Click it for the full map, which shows what you know of the world or of the site you're in.
- **The system menu:** Settings, Logout (not available yet) and Quit. See below.

## Pausing

The system menu opens with Esc or the gear button in the top-right corner. Whether the game pauses while it is open depends on the server:

| Playing on | While the system menu is open |
| ---------- | ------------------------------ |
| Single player (the server built into the native client) | The game pauses: the world's clock stops, and so does anything going by it. "Paused" shows in the menu. |
| A networked server, and so every browser | Nothing pauses. The world carries on, the keys don't reach the game, and a journey you clicked on keeps going. |

So pausing needs both of these: a server that allows it, which only the built-in one does, and the system menu being open. A networked server never pauses, for anyone, whatever its settings. The game carries on as soon as the menu is closed.
