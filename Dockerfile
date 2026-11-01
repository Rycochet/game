# syntax=docker/dockerfile:1
# Hall of Worlds server image: the game server, which also serves the web
# client, on one port.
#
#   docker build -t hall-of-worlds .
#   docker run -p 8080:8080 -v hall-of-worlds:/data hall-of-worlds
#
# Port 8080 serves the web client's files and the game's WebSocket (`/ws`),
# so one HTTP(S) port (or a proxy in front of it) is all it needs. Saves go
# in /data: mount a volume there to keep them.
#
# x86-64 only for now; ARM can be added later as another platform under the
# same tag.

# --- Tools: built once and cached as their own layer -----------------------
FROM rust:1.99-trixie AS tools
RUN cargo install --locked trunk --version 0.22.1

# --- Build the server and the web client ------------------------------------
FROM rust:1.99-trixie AS build
COPY --from=tools /usr/local/cargo/bin/trunk /usr/local/cargo/bin/trunk
WORKDIR /src
COPY . .
# rust-toolchain.toml installs the pinned toolchain and the wasm target.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release --locked --package server --bin hall-of-worlds-server \
 && cp target/release/hall-of-worlds-server /usr/local/bin/ \
 && trunk build --release --dist /srv/web

# --- Runtime -----------------------------------------------------------------
FROM debian:trixie-slim
COPY --from=build /usr/local/bin/hall-of-worlds-server /usr/local/bin/
COPY --from=build /srv/web /srv/web
RUN useradd --system --no-create-home hall \
 && mkdir /data \
 && chown hall /data
USER hall
ENV HOW_ADDRESS=0.0.0.0:8080 \
    HOW_WEB=/srv/web \
    HOW_DATA=/data
VOLUME /data
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/hall-of-worlds-server"]
