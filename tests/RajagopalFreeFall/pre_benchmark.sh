#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Start FooBar pre-benchmark."
echo "    PATH=$PATH"
echo "    LD_LIBRARY_PATH=$LD_LIBRARY_PATH"
echo "    Opensim version = $(opensim-cmd --version)"
echo "Completed FooBar pre-benchmark."
echo

models="../opensim-models"

cp -r $models/Geometry .
cp -r $models/Models/RajagopalModel/Geometry .
cp -r $models/Models/RajagopalModel/Rajagopal2015.osim .
