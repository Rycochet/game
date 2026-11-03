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
| Behind a reverse proxy, unencrypted between it and this server | `HOW_BEHIND_PROXY: "true"` and `HOW_DISABLE_SSL: "true"` | `"8080:8080"` alone |
| Behind a reverse proxy, encrypted between it and this server | Any certificate above or the self-signed one, with `HOW_BEHIND_PROXY: "true"` if it should be able to use port 8080 too | `"8081:8081"`, and `"8080:8080"` if used |

- **Let's Encrypt:** the domain's public port 443 must reach this server, directly rather than through a proxy. The self-signed certificate is used until the new one arrives, with no restart needed.
- **Your own certificate:** replaced files are picked up within an hour, with no restart needed.
- **A reverse proxy:** it must pass WebSockets. Cloudflare's does.

These can't be mixed: settings that disagree stop the server from starting, and it says why.

## Connecting a native client

Players choose **Connect...** on the startup menu and type the server's address, such as `game.example.com` or `game.example.com:9000`. Without a port, it is 8081. The client remembers the servers it has joined, and Play goes back to the last one.

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
- **A networked server never pauses.** The system menu doesn't stop the world for anyone, whatever the settings: see [Pausing](../client#pausing).
- **Time stops only when the server does.** The world's clock is saved, and carries on from there next time.
- **Keep `HOW_DEBUG_TOOLS` off on a public server.** It lets anyone connected use the server's side of the [debug menu](../debug-menu).
- **Stopping:** the server saves everyone when asked to stop (`docker compose down`, Ctrl+C or SIGTERM).
