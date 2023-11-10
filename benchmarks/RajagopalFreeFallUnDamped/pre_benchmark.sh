#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Start RajagopalFreeFall (Opensim version = $(opensim-cmd --version))."

models="$OSPC_OPENSIM_MODELS"

cp -r $models/Geometry .
cp -r $models/Models/RajagopalModel/Geometry .
cp $models/Models/RajagopalModel/Rajagopal2015.osim .

sed -i -e s/'fiber_damping>0.1<\/fiber_damping'/'fiber_damping>0.01<\/fiber_damping'/g Rajagopal2015.osim
