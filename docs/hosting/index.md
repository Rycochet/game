---
title: Hosting a server
nav_order: 5
permalink: /hosting/
---

# Hosting a server

A server runs one world and serves both the game and the web client, so one port is all it needs. The published Docker image is the simplest way to run one. It is x86-64 only for now.

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

Open `https://<your server>:8081` to play. The server makes its own certificate on first starting, and keeps it in the volume with the saves. No browser trusts a certificate like that, so each player has to accept a warning once.

For a certificate that browsers trust without a warning, see the other options below.

To update, which may wipe saves while the game is at this stage (see [Saves and data](../saves)):

```bash
docker compose pull
```

```bash
docker compose up -d
```

## Other ways to secure it

Change the `environment` and `ports` of the example:

| Setup | Environment | Ports |
| ----- | ----------- | ----- |
| Let's Encrypt | `HOW_LETS_ENCRYPT_DOMAIN: game.example.com`, which accepts Let's Encrypt's terms | `"443:8081"` |
| Your own certificate | `HOW_TLS_CERTIFICATE` and `HOW_TLS_KEY`, the paths of your PEM files as the container sees them, which must be readable by its `hall` user | `"8081:8081"` |
| Behind a reverse proxy that looks after TLS | `HOW_BEHIND_PROXY: "true"` | `"8080:8080"` alone |

- **Let's Encrypt:** the domain's public port 443 must reach this server, directly rather than through a proxy. The self-signed certificate is used until the new one arrives, with no restart needed.
- **Your own certificate:** replaced files are picked up within an hour, with no restart needed.
- **A reverse proxy:** it must pass WebSockets. Cloudflare's does.

These can't be mixed: settings that disagree stop the server from starting, and it says why.

## Connecting a native client

Set `HOW_SERVER` when starting the client, using `wss://` and the path `/ws`:

```bash
HOW_SERVER=wss://game.example.com:8081/ws hall-of-worlds
```

There is no server list in the game yet. A certificate from a recognised authority is simply checked. With a self-signed one, the client remembers the server's key on its first visit and refuses if it ever changes. That first visit can't tell the server from an impostor, so to be sure, set `HOW_SERVER_KEY` to the key fingerprint, which is in the server's log when it starts:

```text
Listening securely on 0.0.0.0:8081: key fingerprint ...
```

If a server's key changed for a good reason, remove its line from `known_servers.ron` (see [Saves and data](../saves)).

## Things to know

- **Players are guests for now.** There are no logins yet, and anyone who can reach the server can play. A guest gets their own character, tied to that browser or native client.
- **A networked server never pauses.** The system menu doesn't stop the world for anyone, whatever the settings: see [Pausing](../client#pausing).
- **Time stops only when the server does.** The world's clock is saved, and carries on from there next time.
- **Keep `HOW_DEBUG_TOOLS` off on a public server.** It lets anyone connected use the server's side of the [debug menu](../debug-menu).
- **Stopping:** the server saves everyone when asked to stop (`docker compose down`, Ctrl+C or SIGTERM).
