#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

FRONTEND_PID=0
function my_cleanup() {
    docker compose down mongodb
    if [ $FRONTEND_PID -ne 0 ]; then
        kill $FRONTEND_PID
    fi
}

# If the user specified, use release mode
DO_RELEASE=false
if [ "$1" == "--release" ]; then
    DO_RELEASE=true
fi

trap my_cleanup SIGINT
docker compose up --build -d mongodb --force-recreate
pushd frontend
RELEASE_FLAG=""
if $DO_RELEASE; then
    RELEASE_FLAG="--release"
fi
trunk serve $RELEASE_FLAG --proxy-backend=http://localhost:8081/api/v1 &
FRONTEND_PID=$!
popd
pushd server
cargo watch -- cargo run --bin jnickg_tile_server $RELEASE_FLAG -- --host localhost --user admin --pass ../secrets/mongo-pw.txt --db-port 27017 --port 8081 --static-dir ../dist/
