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
| A browser's guest token, logins and preferences | The browser's local storage for that server's address |

### In a server's folder

- `server.ron`: the server's own clock, which every world's time goes by.
- `worlds/`: each world, its time, the sites in it that have been entered, what players have changed or left in its wilderness, and its roads (`roads/`, each worked out once and kept; safe to delete, as they are worked out again).
- `characters/`: what each character knows, and where they are.
- `accounts/` and `logins/`: who owns which characters, and in `accounts/names.ron` every character's name, so none is used twice. No password, token or email address is stored, only keys made from them.
- `roles.ron`: which accounts have which [role](hosting/#roles), and what each role may do. Edit it only while the server is stopped. `roles-backups/` keeps its last 10 versions, and `audit.log` lists every role given, taken or run out.
- `owner-claim.txt`: the code that claims the server, while it has no Owner. It goes once the server is claimed.
- `login-secret.txt`: what email addresses are made into keys with, if players log in through a provider. Losing it loses every account that logs in that way, and anyone with it and your saves can check whether an address has an account here, so back it up and keep it to yourself.
- `tls/`: its certificates (single player's too, once Online has been on). **`tls/key.pem` is its private key**, so keep it to yourself.
- `save_format.ron`: which format the saves here are in.
- `web-history/`: the last few builds of the web client this server served, and what changed between them and the current one, so a browser that has an older build downloads only that. Only on a server serving the web client; safe to delete, at the cost of whole downloads after the next update.
- `old-saves-<date>/`: saves from an older version that can't be used any more. A new version that wipes saves moves `worlds/`, `characters/`, `accounts/` and `logins/` here when it starts, and leaves everything else, such as certificates and roles, in place. This version can't read them, so delete the folder whenever you like.

### In the native client's folder

- `client.ron`: choices kept between runs, such as the interface size and "Enable Touch" from Settings, the page Item Pictures last showed, the servers joined with Connect (and any certificate pasted in for them), Online with its Server Options, and the window: where it was, its size, and how it was shown (Settings, Graphics). The path can be changed with `HOW_PREFERENCES`.
- `guest_tokens.ron`: the secret that makes you the same guest on each server, beside `client.ron`. Anyone with it can play as you.
- `logins.ron`: for each server you have logged in to with an account, the token that logs you in again and the account's email address, beside `client.ron`. Anyone with it can play as you there until it runs out. Change User on Character Select removes that server's, and so does forgetting the server in Connect... (or, in a browser, Logout on the startup menu for the page's server).

Clearing a browser's data for a server's address, or losing `guest_tokens.ron`, loses that guest's character, as a guest has no other way to log in.

## Backing up

Copy the server's folder while the server is stopped. Saves are written often while it runs, every few seconds, and everything is saved when it stops. In Docker, back up the volume.

If a site, character or account file makes no sense, the server renames it to end in `.ron.broken` and starts that one again, so nothing is lost for good.

If a file can't be reached at all, because of its permissions or a failing disk, it is left alone, and the player is told to try again soon. If saves can't be written, the server keeps trying while it runs, and logs an error. Stopping waits up to 8 seconds for them.
