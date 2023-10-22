#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Start RajagopalContact pre-benchmark script."
echo "    Opensim version = $(opensim-cmd --version)"

cp -r ../../tests/opensim-models/Geometry .
cp -r ../../tests/opensim-models/Models/RajagopalModel/Geometry .
