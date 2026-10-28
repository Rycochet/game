#!/bin/bash
# Runs the web client's static file server and the game server side by side.
# If either stops, the other is stopped too so the container exits (and can be
# restarted by Docker). `docker stop` stops both cleanly.
set -u

static-web-server --host 0.0.0.0 --port 8080 --root /srv/web &
web=$!

hall-of-worlds-server &
game=$!

stop() {
    kill "$web" "$game" 2>/dev/null
}
trap stop TERM INT

# Wait for the first one to exit, then stop the other.
wait -n
status=$?
stop
wait
exit "$status"
