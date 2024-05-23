#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

(trap 'kill 0' SIGINT; \
 bash -c 'cd frontend; trunk serve --proxy-backend=http://localhost:8081/api/v1' & \
 bash -c 'cargo watch -- cargo run --bin jnickg_tile_server -- --host localhost --user admin --pass ./server/secrets/mongo-pw.txt --db-port 27017 --port 8081 --static-dir dist/')


