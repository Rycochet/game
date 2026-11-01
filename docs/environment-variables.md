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
| `HOW_BEHIND_PROXY` | `false` | `true` turns TLS off and serves everything on the plain port, for a server behind a reverse proxy that looks after TLS. Can't be used with a certificate of your own or Let's Encrypt. |
| `HOW_WEB` | none (`/srv/web` in Docker) | The folder holding the web client. Without it, only the game itself is served. |

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
| `HOW_DEBUG_LETS_ENCRYPT_STAGING` | `false` | `true` uses Let's Encrypt's test service, whose certificates nobody trusts, to try things without its rate limits. |

## For the native client

The native client reads `HOW_DATA` too, for its single-player saves, as well as these. They have no effect on a server.

| Variable | Default | What it does |
| -------- | ------- | ------------ |
| `HOW_SERVER` | none | The server to join, such as `wss://game.example.com:8081/ws`. Without it, the client plays single player. |
| `HOW_SERVER_KEY` | none | The key fingerprint the server must have, from its log. See [Connecting a native client](hosting/#connecting-a-native-client). |
| `HOW_KNOWN_SERVERS` | `Hall of Worlds/client/known_servers.ron` in the user's data folder | The file remembering the servers this client has visited. |
