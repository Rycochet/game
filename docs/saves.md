---
title: Saves and data
nav_order: 8
---

# Saves and data

> **While the game is at this early stage, new versions may wipe saves.**

## Where things are kept

The user's data folder is `%APPDATA%` on Windows, `~/Library/Application Support` on macOS and `~/.local/share` on Linux.

| What | Where |
| ---- | ----- |
| A server's saves, and single player's | `Hall of Worlds/server` in the user's data folder, or wherever `HOW_DATA` says |
| In Docker | `/data`, which should be a volume |
| The native client's own files | `Hall of Worlds/client` in the user's data folder |
| A browser's guest token | The browser's local storage for that server's address |

### In a server's folder

- `worlds/`: each world, and the sites in it that have been entered.
- `characters/`: what each character knows, and where they are.
- `accounts/` and `logins/`: who owns which characters. No login secret is stored.
- `tls/`: its certificates. **`tls/key.pem` is its private key**, so keep it to yourself.

### In the native client's folder

- `known_servers.ron`: the servers visited and how each is trusted. The path can be changed with `HOW_KNOWN_SERVERS`.
- `guest_tokens.ron`: the secret that makes you the same guest on each server. Anyone with it can play as you.

Clearing a browser's data for a server's address, or losing `guest_tokens.ron`, loses that guest's character, as there's no other way to log in yet.

## Backing up

Copy the server's folder while the server is stopped. Saves are written often while it runs, every few seconds, and everything is saved when it stops. In Docker, back up the volume.

If a site or character file can't be read, the server renames it to end in `.ron.broken` and starts that one again, so nothing is lost for good.
