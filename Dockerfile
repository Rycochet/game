# syntax=docker/dockerfile:1
# Hall of Worlds server image: the game server plus the web client.
#
#   docker build -t hall-of-worlds .
#   docker run -p 8080:8080 hall-of-worlds
#
# Port 8080 serves the web client (static files). The game server's own port
# is added when networking lands.
#
# x86-64 only for now; ARM can be added later as another platform under the
# same tag.

# --- Tools: built once and cached as their own layer -----------------------
FROM rust:1.99-trixie AS tools
RUN cargo install --locked trunk --version 0.22.1 \
 && cargo install --locked static-web-server --version 2.44.1

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
COPY --from=tools /usr/local/cargo/bin/static-web-server /usr/local/bin/
COPY --from=build /usr/local/bin/hall-of-worlds-server /usr/local/bin/
COPY --from=build /srv/web /srv/web
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh \
 && useradd --system --no-create-home hall
USER hall
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
