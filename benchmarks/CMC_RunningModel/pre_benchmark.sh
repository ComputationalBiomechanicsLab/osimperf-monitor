#!/bin/bash

# Exit immediately if any command exits with a non-zero status
set -e
# Exit with a non-zero status if any command in a pipeline fails
set -o pipefail

echo
echo "Start RunningCMC (Opensim version = $(opensim-cmd --version))."

models="$OSPC_OPENSIM_MODELS"

if [ -z $models ]; then
	echo "ERROR: no models dir found"
	exit 1
fi

cp -r $models/Geometry .
cp $models/Models/RajagopalModel/Geometry/*.vtp Geometry/
