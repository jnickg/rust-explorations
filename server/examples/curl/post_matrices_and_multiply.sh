#!/bin/bash

pushd "$(dirname "$0")"

pushd ../..

cargo run  &
PID=$!
sleep 2 # hacky but probably good enough

popd # back to this directory

curl --data "@2x2ident.json" -H "Content-Type: application/json" -X POST http://localhost:3000/api/v1/matrix/2x2ident
curl --data "@3456.json" -H "Content-Type: application/json" -X POST http://localhost:3000/api/v1/matrix/3456

curl -X POST http://localhost:3000/api/v1/matrix/multiply/3456/2x2ident > /tmp/response.json

echo ""
echo ""
echo ""
echo "Multiplied $(cat ./3456.json) and $(cat ./2x2ident.json) and got result: $(cat /tmp/response.json)"

kill $PID
rm /tmp/response.json

popd
