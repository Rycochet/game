# syntax=docker/dockerfile:1
# Hall of Worlds server image: the game server, which also serves the web
# client, on one port.
#
#   docker build -t hall-of-worlds .
#   docker run -p 24691:24691 -p 24690:24690 -v hall-of-worlds:/data hall-of-worlds
#
# Port 24691 serves the web client's files and the game's WebSocket (`/ws`) over
# HTTPS, with a self-signed certificate kept in /data unless HOW_TLS_CERTIFICATE
# and HOW_TLS_KEY name your own (readable by the server's user, below), or
# HOW_LETS_ENCRYPT_DOMAIN asks Let's Encrypt for one (publish it as -p 443:24691).
# Port 24690 only answers that connections must be secure.
#
# Behind a reverse proxy, set HOW_BEHIND_PROXY=true: port 24690 then serves
# everything too, and 24691 stays open in case the proxy prefers it. HOW_DISABLE_SSL=true
# as well doesn't open 24691 or set up a certificate:
#
#   docker run -p 24690:24690 -e HOW_BEHIND_PROXY=true -e HOW_DISABLE_SSL=true -v hall-of-worlds:/data hall-of-worlds
#
# Saves go in /data: mount a volume there to keep them.
#
# The server runs as user and group 1000:1000 unless PUID and PGID say
# otherwise, given when building (--build-arg) or when starting (-e). Started
# as root, as by default, the image first gives anything in /data that isn't
# theirs to them. Started with --user (Compose's `user:`), it runs as that user
# and leaves /data's owner to you.
#
# The web client is at /web/ (where / sends browsers). Every port also answers
# /_healthz (200 while the game runs) and /_version (what the server is, as
# JSON). Docker's health check asks the server's own binary, which needs no
# curl or wget in the image.
#
# x86-64 only for now; ARM can be added later as another platform under the
# same tag.
#
# This file builds everything inside Docker, for building the image yourself.
# GitHub's Docker workflow uses Dockerfile.github instead, which only copies in
# what the workflow compiled outside Docker (where its cache works). Both must
# make the same image: the part after the "Same in" line below is the same in
# both (`cargo xtask lint` checks it), and a change to the build steps here
# needs the same change in .github/workflows/docker.yml.

# --- Tools: built once and cached as their own layer -----------------------
FROM rust:1.99-trixie AS tools
RUN cargo install --locked trunk --version 0.22.1

# --- Build the server and the web client ------------------------------------
FROM rust:1.99-trixie AS build
COPY --from=tools /usr/local/cargo/bin/trunk /usr/local/cargo/bin/trunk
WORKDIR /src
COPY . .
# rust-toolchain.toml installs the pinned toolchain and the wasm target.
# The web client is built with its own, smaller profile (`web` in Cargo.toml),
# then given brotli copies of its files for browsers that take them.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked --package server --bin hall-of-worlds-server \
 && cp target/release/hall-of-worlds-server /usr/local/bin/ \
 && trunk build --release --cargo-profile web --dist /srv/web \
 && cargo xtask compress-web /srv/web

# --- Runtime -----------------------------------------------------------------
FROM debian:trixie-slim
COPY --from=build /usr/local/bin/hall-of-worlds-server /usr/local/bin/
COPY --from=build /srv/web /srv/web
# --- Same in Dockerfile and Dockerfile.github from here to the end ------------
# Certificate authorities: the server checks Let's Encrypt's website with them.
RUN apt-get update \
 && apt-get install --yes --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
# The user and group the server runs as: PUID and PGID, when building or when
# starting (see entrypoint.sh below). setpriv comes with Debian (util-linux).
ARG PUID=1000
ARG PGID=1000
RUN groupadd --non-unique --gid "$PGID" hall \
 && useradd --non-unique --uid "$PUID" --gid hall --no-create-home --shell /usr/sbin/nologin hall \
 && mkdir /data \
 && chown hall:hall /data \
 && command -v setpriv
# Started as root, it gives /data to PUID:PGID (only what isn't theirs already)
# and then runs the server as them. Started as another user (Docker's `user:`
# or --user), it leaves /data alone and runs the server as that user, stopping
# with a message if /data can't be written. Either way the server replaces the
# script (exec), so it gets Docker's stop signal and saves everyone.
COPY --chmod=755 <<'EOF' /usr/local/bin/entrypoint.sh
#!/bin/sh
set -eu
server=/usr/local/bin/hall-of-worlds-server
data="${HOW_DATA:-/data}"
if [ "$(id -u)" != 0 ]; then
    if [ ! -w "$data" ]; then
        echo "$data can't be written by user $(id -u):$(id -g). Give it to that user (chown -R $(id -u):$(id -g) on the volume), or leave out user: and set PUID and PGID instead." >&2
        exit 1
    fi
    exec "$server" "$@"
fi
for id in "$PUID" "$PGID"; do
    case "$id" in
        '' | *[!0-9]*)
            echo "PUID and PGID must be numbers, not '$PUID' and '$PGID'." >&2
            exit 1
            ;;
    esac
done
mkdir -p "$data"
if ! find "$data" \( ! -user "$PUID" -o ! -group "$PGID" \) -exec chown -h "$PUID:$PGID" {} +; then
    echo "Some files in $data couldn't be given to $PUID:$PGID." >&2
fi
exec setpriv --reuid="$PUID" --regid="$PGID" --clear-groups -- "$server" "$@"
EOF
ENV PUID=$PUID \
    PGID=$PGID \
    HOW_ADDRESS=0.0.0.0:24690 \
    HOW_WEB=/srv/web \
    HOW_DATA=/data
VOLUME /data
EXPOSE 24690 24691
# Asks /_healthz on the plain port (HOW_ADDRESS), which answers even while
# only the secure port serves the game. Making a new world takes a while.
HEALTHCHECK --interval=30s --timeout=5s --start-period=60s --retries=3 \
    CMD ["/usr/local/bin/hall-of-worlds-server", "--health"]
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
