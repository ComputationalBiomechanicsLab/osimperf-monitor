#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Start RajagopalContact (Opensim version = $(opensim-cmd --version))."

models="$OSPC_OPENSIM_MODELS"

cp -r $models/Geometry .
cp -r $models/Models/RajagopalModel/Geometry .
