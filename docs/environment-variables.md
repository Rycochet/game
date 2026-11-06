---
title: Server environment variables
nav_order: 6
---

# Server environment variables

A server is set up entirely by environment variables, and none of them are required. Yes-or-no options take `true` or `false`.

## Network

| Variable | Default | What it does |
| -------- | ------- | ------------ |
| `HOW_ADDRESS` | `0.0.0.0:8080` | The plain port's address. It only answers that connections must be secure, unless `HOW_BEHIND_PROXY` is on. |
| `HOW_TLS_ADDRESS` | `0.0.0.0:8081` | The secure port's address, for HTTPS and `wss://`. |
| `HOW_BEHIND_PROXY` | `false` | `true` also serves everything on the plain port, for a reverse proxy to reach. The secure port stays open, so the proxy can use either. |
| `HOW_DISABLE_SSL` | `false` | `true` doesn't open the secure port or set up a certificate. The server refuses to start unless `HOW_BEHIND_PROXY` is on too, and if a certificate of your own or Let's Encrypt is set. |
| `HOW_WEB` | none (`/srv/web` in Docker) | The folder holding the web client. Without it, only the game itself is served. The last few builds it served are kept in the data folder, so browsers that have one download only what changed. |

## Certificates

Without any of these, the server makes a self-signed certificate and keeps it in the data folder.

| Variable | Default | What it does |
| -------- | ------- | ------------ |
| `HOW_TLS_NAMES` | none | More names and addresses, separated by commas, for a certificate the server makes itself. For example `game.example.com,192.168.0.10`. |
| `HOW_TLS_CERTIFICATE` | none | The path of your own PEM certificate. Needs `HOW_TLS_KEY` too. |
| `HOW_TLS_KEY` | none | The path of your own PEM private key. Needs `HOW_TLS_CERTIFICATE` too. |
| `HOW_LETS_ENCRYPT_DOMAIN` | none | Get a certificate for this domain from Let's Encrypt, which accepts its terms. The domain's public port 443 must reach the secure port. Can't be used with your own certificate. |
| `HOW_LETS_ENCRYPT_EMAIL` | none | Where Let's Encrypt can write to you. Needs `HOW_LETS_ENCRYPT_DOMAIN`. |

## Game and saves

| Variable | Default | What it does |
| -------- | ------- | ------------ |
| `HOW_DATA` | `Hall of Worlds/server` in the user's data folder (`/data` in Docker) | Where to save. See [Saves and data](saves). |
| `HOW_GUESTS` | `true` | Whether people can play without an account. `false` stops the server from starting for now, as there is no other way to log in yet. |
| `HOW_SERVER_NAME` | `Hall of Worlds` | What the server calls itself, for instance when telling players it is going offline. |
| `HOW_PLAYER_LIMIT` | `0` | The most players at once; anyone more is told the server is full. `0` is no limit. |
| `HOW_STACK_GRAMS_PER_CELL` | `2000` | The most a stack of identical items may weigh, in grams, for each cell it takes up in a bag. |
| `HOW_STACK_MAX` | `99` | The most items in one stack, however light they are. |

## Logging

| Variable | Default | What it does |
| -------- | ------- | ------------ |
| `LOG_LEVEL` | `info` | How much to log: `error`, `warn`, `info`, `debug` or `trace`. `debug` adds a line a second for each connection's round trip, for looking into network trouble. |
| `RUST_LOG` | none | Filters the log by crate, overriding `LOG_LEVEL`. For example `server=debug,info`. |

## Debugging

A real server can ignore these.

| Variable | Default | What it does |
| -------- | ------- | ------------ |
| `HOW_DEBUG_TOOLS` | `false` | `true` accepts developer tool messages from **anyone connected**, for private test servers. It turns on the entries of the [debug menu](debug-menu) that need the server. |
| `HOW_DEBUG_API` | `false` | Only in development builds made with `--features debug-api`, never in releases or the Docker image. `true` starts the debug API on port 15702, or give a port number; it only listens on this computer (127.0.0.1). The native client's own single-player server takes the next port. If the port is taken, the game logs it and starts without the API. |
| `HOW_DEBUG_GENERATOR` | none | `Desert`, `Continents` or `Plates` makes every world's land that way instead of its own, to try one out. A client must have the same setting to play there, as the native client's own server does. |
| `HOW_DEBUG_LETS_ENCRYPT_STAGING` | `false` | `true` uses Let's Encrypt's test service, whose certificates nobody trusts, to try things without its rate limits. |

## For the native client

The native client reads `HOW_DATA` too, for its single-player saves, as well as these. They have no effect on a server.

| Variable | Default | What it does |
| -------- | ------- | ------------ |
| `HOW_SERVER` | none | A server for Play to join, such as `wss://game.example.com:8081/ws`, until logging out; a server chosen in Connect comes first. Without either, Play is single player. |
| `HOW_SERVER_KEY` | none | The key fingerprint `HOW_SERVER` must have, from its log. Without it, any certificate is accepted. See [Connecting a native client](hosting/#connecting-a-native-client). |
| `HOW_PREFERENCES` | `Hall of Worlds/client/client.ron` in the user's data folder | The file keeping the player's choices between runs. |
