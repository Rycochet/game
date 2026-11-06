---
title: Hosting a server
nav_order: 5
permalink: /hosting/
---

# Hosting a server

A server runs every world its players are in, and serves both the game and the web client, so one port is all it needs. The published Docker image is the simplest way to run one. It is x86-64 only for now.

Every setting is listed under [Server environment variables](../environment-variables).

## Docker Compose, with a self-signed certificate

Save this as [`docker-compose.yml`](docker-compose.yml):

```yaml
{% include_relative docker-compose.yml %}
```

Then start it:

```bash
docker compose up -d
```

Open `https://<your server>:8081` to play: it goes to the web client, at `/web/`. The server makes its own certificate on first starting, and keeps it in the volume with the saves. No browser trusts a certificate like that, so each player has to accept a warning once.

For a certificate that browsers trust without a warning, see the other options below.

To update, which may wipe saves while the game is at this stage (see [Saves and data](../saves)):

```bash
docker compose pull
```

```bash
docker compose up -d
```

The `latest` image changes every time something is added to the game, which can be several times a day. For something steadier, use `nightly` in its place: it changes at most once a day. `nightly-<date>` (such as `nightly-2026-11-06`) keeps a particular day's.

## Other ways to secure it

Change the `environment` and `ports` of the example:

| Setup | Environment | Ports |
| ----- | ----------- | ----- |
| Let's Encrypt | `HOW_LETS_ENCRYPT_DOMAIN: game.example.com`, which accepts Let's Encrypt's terms | `"443:8081"` |
| Your own certificate | `HOW_TLS_CERTIFICATE` and `HOW_TLS_KEY`, the paths of your PEM files as the container sees them, which must be readable by its `hall` user | `"8081:8081"` |
| Behind a reverse proxy, unencrypted between it and this server | `HOW_BEHIND_PROXY: "true"` and `HOW_DISABLE_SSL: "true"` | `"8080:8080"` alone |
| Behind a reverse proxy, encrypted between it and this server | Any certificate above or the self-signed one, with `HOW_BEHIND_PROXY: "true"` if it should be able to use port 8080 too | `"8081:8081"`, and `"8080:8080"` if used |

- **Let's Encrypt:** the domain's public port 443 must reach this server, directly rather than through a proxy. The self-signed certificate is used until the new one arrives, with no restart needed.
- **Your own certificate:** replaced files are picked up within an hour, with no restart needed.
- **A reverse proxy:** it must pass WebSockets. Cloudflare's does. The web client's files come already compressed, and say how long they may be kept, so a proxy that caches needs no rules of its own for them. After an update, browsers that have an earlier build download only what changed, if the proxy passes compression dictionaries through (Cloudflare can).

These can't be mixed: settings that disagree stop the server from starting, and it says why.

## Connecting a native client

Players choose **Connect...** on the startup menu and type the server's address, such as `game.example.com` or `game.example.com:8081`. Without a port, the client tries 8081 and then 443, so a server on either needs no port typed; it asks `/_version` to tell a Hall of Worlds server from any other web server. The client remembers the servers it has joined, and Play goes back to the last one.

The client accepts any certificate the server has, so you can change yours whenever you like. A player who wants to be sure they've reached your server, and not an impostor, can paste its certificate (from `https://<your server>:8081/certificate`) or its key's fingerprint into the padlock beside the address. From then on, only a server with that key is accepted, and the client says plainly why if it isn't. The fingerprint is in the server's log when it starts:

```text
Listening securely on 0.0.0.0:8081: key fingerprint ...
```

If you change your key, tell players who pasted the old one, so they can paste the new one.

For testing, `HOW_SERVER` starts the client on a server straight away, using `wss://` and the path `/ws`, and `HOW_SERVER_KEY` gives the key it must have:

```bash
HOW_SERVER=wss://game.example.com:8081/ws hall-of-worlds
```

## Things to know

- **Players are guests for now.** There are no logins yet, and anyone who can reach the server can play. A guest gets their own character, tied to that browser or native client.
- **A networked server never pauses.** The system menu (or a player's window going into the background) doesn't stop the world for anyone, whatever the settings, and nor does a native client's own server while Online is on: see [Pausing](../client#pausing).
- **Hosting from the game itself:** a native client can open its own world to others with Online in the system menu, on one secure port (8081 unless Server Options says otherwise), with a certificate it makes. Players on the internet need that port forwarded to the computer. A standalone server is better for a world that is always there.
- **Time stops only when the server does.** The server's clock is saved, and carries on from there next time. Every world's time goes by it, so a world nobody is in keeps its time going too.
- **Keep `HOW_DEBUG_TOOLS` off on a public server.** It lets anyone connected use the server's side of the [debug menu](../debug-menu).
- **Stopping:** the server saves everyone when asked to stop (`docker compose down`, Ctrl+C or SIGTERM).
- **Health checks:** every port answers `/_healthz` with `200 ok` while the game is running, and `503` if it has stopped, even the plain port that otherwise only says to use the secure one. The Docker image checks it by itself (`docker ps` shows `healthy`), asking with the server's own `hall-of-worlds-server --health`, which needs no curl or wget. `/_version` says what the server is, as JSON, such as `{"game":"Hall of Worlds","version":"0.1.0","protocol":37}`.
