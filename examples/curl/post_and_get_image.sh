#!/bin/bash

# This script requires ImageMagick to be installed
if ! command -v identify &> /dev/null
then
    echo "identify could not be found"
    exit
fi


pushd "$(dirname "$0")"

pushd ../..

cargo run  &
PID=$!
sleep 2 # hacky but probably good enough

popd # back to this directory

curl --data-binary "@helldivers.jpg" -H "Content-Type: image/jpeg" -X POST http://localhost:3000/api/v1/image 
curl -X GET http://localhost:3000/api/v1/image/image_0 --output /tmp/helldivers.png
curl -H "Accept: image/jpeg" -X GET http://localhost:3000/api/v1/image/image_0 --output /tmp/helldivers.jpg

echo ""
echo ""
echo ""
identify /tmp/helldivers.png
identify /tmp/helldivers.jpg

kill $PID
rm /tmp/helldivers.png
rm /tmp/helldivers.jpg

popd


