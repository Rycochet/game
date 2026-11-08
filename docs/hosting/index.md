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

Open `https://<your server>:24691` to play: it goes to the web client, at `/web/`. The server makes its own certificate on first starting, and keeps it in the volume with the saves. No browser trusts a certificate like that, so each player has to accept a warning once.

For a certificate that browsers trust without a warning, see the other options below.

To update, which may wipe saves while the game is at this stage (see [Saves and data](../saves)):

```bash
docker compose pull
```

```bash
docker compose up -d
```

Before 7 November 2026 the ports were 8080 and 8081. If you update from then, change both sides of `ports` in your `docker-compose.yml` (and anything in front of the server, such as a proxy's or a tunnel's origin) to 24690 and 24691, or set `HOW_ADDRESS` and `HOW_TLS_ADDRESS` to keep the old ones.

The `latest` image changes every time something is added to the game, which can be several times a day. For something steadier, use `nightly` in its place: it changes at most once a day. `nightly-<date>` (such as `nightly-2026-11-06`) keeps a particular day's. Players whose browsers last played on any of the past week's nightly images download only what has changed in the web client, not all of it.

## Other ways to secure it

Change the `environment` and `ports` of the example:

| Setup | Environment | Ports |
| ----- | ----------- | ----- |
| Let's Encrypt | `HOW_LETS_ENCRYPT_DOMAIN: game.example.com`, which accepts Let's Encrypt's terms | `"443:24691"` |
| Your own certificate | `HOW_TLS_CERTIFICATE` and `HOW_TLS_KEY`, the paths of your PEM files as the container sees them, which must be readable by its `hall` user | `"24691:24691"` |
| Behind a reverse proxy, unencrypted between it and this server | `HOW_BEHIND_PROXY: "true"` and `HOW_DISABLE_SSL: "true"` | `"24690:24690"` alone |
| Behind a reverse proxy, encrypted between it and this server | Any certificate above or the self-signed one, with `HOW_BEHIND_PROXY: "true"` if it should be able to use port 24690 too | `"24691:24691"`, and `"24690:24690"` if used |

- **Let's Encrypt:** the domain's public port 443 must reach this server, directly rather than through a proxy. The self-signed certificate is used until the new one arrives, with no restart needed.
- **Your own certificate:** replaced files are picked up within an hour, with no restart needed.
- **A reverse proxy:** it must pass WebSockets. Cloudflare's does. The web client's files come already compressed, and say how long they may be kept, so a proxy that caches needs no rules of its own for them. After an update, browsers that have an earlier build download only what changed, if the proxy passes compression dictionaries through (Cloudflare can).

These can't be mixed: settings that disagree stop the server from starting, and it says why.

With none of these behind a proxy, `HOW_UPNP: "true"` asks your router to forward the port for you (UPnP), if it allows that: 443 with Let's Encrypt, otherwise 24691. The log says whether it worked. In Docker this only works with `network_mode: host` in place of `ports`, as the router's answers can't reach a container on Docker's own network. Behind a proxy it is ignored, with a warning, as the proxy's port is the one to forward.

## Connecting a native client

Players choose **Connect...** on the startup menu and type the server's address, such as `game.example.com` or `game.example.com:24691`. Without a port, the client tries 24691 and then 443, so a server on either needs no port typed; it asks `/_version` to tell a Hall of Worlds server from any other web server. The client remembers the servers it has joined, and Play goes back to the last one.

The client accepts any certificate the server has, so you can change yours whenever you like. A player who wants to be sure they've reached your server, and not an impostor, can paste its certificate (from `https://<your server>:24691/certificate`) or its key's fingerprint into the padlock beside the address. From then on, only a server with that key is accepted, and the client says plainly why if it isn't. The fingerprint is in the server's log when it starts:

```text
Listening securely on 0.0.0.0:24691: key fingerprint ...
```

If you change your key, tell players who pasted the old one, so they can paste the new one.

For testing, `HOW_SERVER` starts the client on a server straight away, using `wss://` and the path `/ws`, and `HOW_SERVER_KEY` gives the key it must have:

```bash
HOW_SERVER=wss://game.example.com:24691/ws hall-of-worlds
```

## Logging in with Google

Players can play as guests, or log in with an account through a provider's website. Google is the one we use; any provider that supports OpenID Connect works the same way.

1. In the Google Cloud console, under **APIs & Services**, set up the OAuth consent screen, then make an **OAuth client ID** of the type **Web application**.
2. Add `https://<your server>/login/callback` as an **authorised redirect URI**, using the address players reach the server at (with its port, if it isn't 443).
3. Give the server the client id and secret, and that address:

```yaml
    environment:
      HOW_OIDC_GOOGLE_CLIENT_ID: "<client id>"
      HOW_OIDC_GOOGLE_CLIENT_SECRET: "<client secret>"
      HOW_LOGIN_URL: "https://game.example.com"
```

With Let's Encrypt, `HOW_LOGIN_URL` can be left out: the domain is used.

To keep the client secret out of the Compose file, give it as a Docker secret and name its file with `HOW_OIDC_GOOGLE_CLIENT_SECRET_FILE`. `HOW_LOGIN_SECRET_FILE` does the same for the login secret.

```yaml
services:
  hall-of-worlds:
    environment:
      HOW_OIDC_GOOGLE_CLIENT_ID: "<client id>"
      HOW_OIDC_GOOGLE_CLIENT_SECRET_FILE: /run/secrets/google_client_secret
    secrets:
      - google_client_secret

secrets:
  google_client_secret:
    file: ./google_client_secret.txt
```

The file must be readable by the container's `hall` user (`chmod 644` on it does that). If the service has `read_only: true`, Compose only allows secrets from a `file:`, as above, as it can't write `environment:` ones into the container; set `HOW_OIDC_GOOGLE_CLIENT_SECRET` itself instead if you'd rather keep them in the environment.

The server only asks Google for the player's email address, and only one Google has checked. It keeps a key made from the address with a secret of its own, never the address itself, and forgets everything else Google says. The secret is made on the first run and kept in the data folder (`login-secret.txt`); servers that share players must be given the same one with `HOW_LOGIN_SECRET`.

If the provider can't be reached when the server starts, or a setting is missing, the server says so in its log and lets players in as guests only. `HOW_GUEST_MODE: "true"` does that on purpose.

Players then choose Log in with Google or Play as Guest when they join. The page opens in their own browser, and once they have logged in there they go back to the game. Their game remembers the login, so they only see that page again after 90 days without playing, or after using Logout on the startup menu. Logins only go over a secure connection, so a server reached through an unsecured port offers guests only.

## Things to know

- **Anyone who can reach the server can play as a guest,** unless `HOW_GUESTS` is `false` and a provider is set up. A guest gets their own character, tied to that browser or native client.
- **A networked server never pauses.** The system menu (or a player's window going into the background) doesn't stop the world for anyone, whatever the settings, and nor does a native client's own server while Online is on: see [Pausing](../client#pausing).
- **Hosting from the game itself:** a native client can open its own world to others with Online in the system menu, on one secure port (24691 unless Server Options says otherwise), with a certificate it makes. With UPnP turned on in Server Options (or `HOW_UPNP=true`), it asks your router to forward that port to the computer while it is online, and says there how that went and the address players on the internet connect to. Otherwise, or if the router has no UPnP or refuses, forward the port yourself. Windows may ask whether the game can use the network the first time: allow it, or the router's answer can't get through. If your internet provider shares one address between many homes, Server Options says so: no forwarding gets players through then. A standalone server is better for a world that is always there.
- **Time stops only when the server does.** The server's clock is saved, and carries on from there next time. Every world's time goes by it, so a world nobody is in keeps its time going too.
- **Keep `HOW_DEBUG_TOOLS` off on a public server.** It lets anyone connected use the server's side of the [debug menu](../debug-menu).
- **Stopping:** the server saves everyone when asked to stop (`docker compose down`, Ctrl+C or SIGTERM).
- **Health checks:** every port answers `/_healthz` with `200 ok` while the game is running, and `503` if it has stopped, even the plain port that otherwise only says to use the secure one. The Docker image checks it by itself (`docker ps` shows `healthy`), asking with the server's own `hall-of-worlds-server --health`, which needs no curl or wget. `/_version` says what the server is, as JSON, such as `{"game":"Hall of Worlds","version":"0.1.0","protocol":37}`.
