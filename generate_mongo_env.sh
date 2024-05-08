#!/bin/bash

if [ ! -f secrets/mongo-pw.txt ] || [ ! -f secrets/mongo-user.txt ]; then
  echo "ERROR: secrets/mongo-pw.txt or secrets/mongo-user.txt not found. Exiting..."
  exit 1
fi

echo "MONGO_INITDB_ROOT_USERNAME=$(cat secrets/mongo-user.txt)" > secrets/mongo-env.txt
echo "MONGO_INITDB_ROOT_PASSWORD=$(cat secrets/mongo-pw.txt)" >> secrets/mongo-env.txt
echo "MONGO_INITDB_DATABASE=tiler" >> secrets/mongo-env.txt
