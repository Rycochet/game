---
title: Installation
nav_order: 2
---

# Installation

There are no public downloads yet. The ways to play are below.

## In a browser

Nothing to install: open the address of a server that serves the web client. The browser version always plays on that server, so it needs one to be running (see [Hosting a server](hosting/)).

## The native client

The native client runs on Windows, macOS and Linux, and includes a server for single player. For now it has to be built from source:

1. Follow the one-time setup in the project's [README](https://github.com/Rycochet/game#one-time-setup-windows).
2. Run `cargo xtask package`. The first build takes a while.
3. Run `hall-of-worlds` from `target/package/`, keeping the `assets` folder beside it.

Single player starts straight away. To join a server instead, see [Connecting a native client](hosting/#connecting-a-native-client).

## A server

The simplest way is the published Docker image, `ghcr.io/rycochet/hall-of-worlds:latest`, which also serves the web client. See [Hosting a server](hosting/).

`cargo xtask package` also builds the server on its own, as `hall-of-worlds-server`.
