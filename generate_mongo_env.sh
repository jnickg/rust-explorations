#!/bin/bash

if [ ! -f server/secrets/mongo-pw.txt ] || [ ! -f server/secrets/mongo-user.txt ]; then
  echo "ERROR: secrets/mongo-pw.txt or secrets/mongo-user.txt not found. Exiting..."
  exit 1
fi

echo "MONGO_INITDB_ROOT_USERNAME=$(cat server/secrets/mongo-user.txt)" > server/secrets/mongo-env.txt
echo "MONGO_INITDB_ROOT_PASSWORD=$(cat server/secrets/mongo-pw.txt)" >> server/secrets/mongo-env.txt
echo "MONGO_INITDB_DATABASE=tiler" >> server/secrets/mongo-env.txt
