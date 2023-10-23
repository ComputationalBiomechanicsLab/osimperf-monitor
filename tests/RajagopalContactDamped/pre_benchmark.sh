#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Start RajagopalContactDamped (Opensim version = $(opensim-cmd --version))."

models="../opensim-models"

cp -r $models/Geometry .
cp -r $models/Models/RajagopalModel/Geometry .
cp ../RajagopalContact/Rajagopal2015.osim .
cp ../RajagopalContact/Rajagopal_setup_forward_tool.xml .

SetFiberDamping Rajagopal2015.osim 0.1
