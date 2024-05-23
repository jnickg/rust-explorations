#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

pushd frontend
trunk build
popd

cargo run --bin server --release -- --port 8080 --static-dir ./dist